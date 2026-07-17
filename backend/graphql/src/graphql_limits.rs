//! Phase 8: Query depth and complexity limits for GraphQL.
//!
//! Parses the query into a real GraphQL AST (rather than brace-counting) so complexity
//! reflects actual field selections. This matters because a brace-counting complexity score
//! only grows with nesting, not with the number of fields selected at a given level: a query
//! that aliases the same expensive field thousands of times in one selection set has the same
//! brace-count as a query with a single field, so it sails through depth/complexity checks that
//! don't count individual field selections. Field-based counting scores each selected field
//! (including aliases and fields reached via fragment spreads) by its nesting depth, so wide
//! shallow queries are penalized proportionally to their width.
//!
//! Complexity is enforced by default now (not opt-in): GRAPHQL_MAX_QUERY_COMPLEXITY only
//! overrides the default, it no longer gates whether the check runs at all.

use graphql_parser::query::{
    Definition, FragmentDefinition, OperationDefinition, ParseError, Selection, SelectionSet,
};
use std::collections::{HashMap, HashSet};

/// Default maximum allowed query depth (configurable via env).
pub const DEFAULT_MAX_QUERY_DEPTH: u32 = 10;

/// Default maximum complexity score (configurable via env).
/// Depth-weighted: each selected field (including aliases) at depth d adds d to the total.
pub const DEFAULT_MAX_QUERY_COMPLEXITY: u64 = 250;

/// Maximum allowed page size for list fields (limit/offset pagination).
pub const MAX_PAGE_SIZE: i64 = 50;

#[derive(Debug, Clone, Copy, Default)]
pub struct QueryMetrics {
    pub depth: u32,
    pub complexity: u64,
}

fn collect_fragments<'a>(
    doc: &'a graphql_parser::query::Document<'a, &'a str>,
) -> HashMap<&'a str, &'a FragmentDefinition<'a, &'a str>> {
    let mut map = HashMap::new();
    for def in &doc.definitions {
        if let Definition::Fragment(frag) = def {
            map.insert(frag.name, frag);
        }
    }
    map
}

#[allow(clippy::too_many_arguments)]
fn walk_selection_set<'a>(
    selection_set: &SelectionSet<'a, &'a str>,
    depth: u32,
    fragments: &HashMap<&'a str, &FragmentDefinition<'a, &'a str>>,
    visiting: &mut HashSet<&'a str>,
    metrics: &mut QueryMetrics,
) {
    if depth > metrics.depth {
        metrics.depth = depth;
    }
    for item in &selection_set.items {
        match item {
            Selection::Field(field) => {
                // Each selected field (including aliases) is charged for its own depth, so
                // N aliases of the same field at depth d cost N*d, not a flat per-brace amount.
                metrics.complexity = metrics.complexity.saturating_add(u64::from(depth));
                if !field.selection_set.items.is_empty() {
                    walk_selection_set(
                        &field.selection_set,
                        depth + 1,
                        fragments,
                        visiting,
                        metrics,
                    );
                }
            }
            Selection::InlineFragment(inline) => {
                // Inline fragments don't add a selection nesting level of their own.
                walk_selection_set(&inline.selection_set, depth, fragments, visiting, metrics);
            }
            Selection::FragmentSpread(spread) => {
                let Some(frag) = fragments.get(spread.fragment_name) else {
                    continue;
                };
                // Guard against fragment cycles (invalid GraphQL, but don't loop forever on it).
                if !visiting.insert(spread.fragment_name) {
                    continue;
                }
                walk_selection_set(&frag.selection_set, depth, fragments, visiting, metrics);
                visiting.remove(spread.fragment_name);
            }
        }
    }
}

fn operation_selection_set<'a, 'b>(
    def: &'b Definition<'a, &'a str>,
) -> Option<&'b SelectionSet<'a, &'a str>> {
    match def {
        Definition::Operation(op) => Some(match op {
            OperationDefinition::SelectionSet(s) => s,
            OperationDefinition::Query(q) => &q.selection_set,
            OperationDefinition::Mutation(m) => &m.selection_set,
            OperationDefinition::Subscription(s) => &s.selection_set,
        }),
        Definition::Fragment(_) => None,
    }
}

/// Parses `query` and computes its real depth/complexity by walking the AST (fields, aliases,
/// inline fragments, and fragment spreads all counted), instead of brace-counting.
pub fn analyze_query(query: &str) -> Result<QueryMetrics, ParseError> {
    let doc = graphql_parser::parse_query::<&str>(query)?;
    let fragments = collect_fragments(&doc);
    let mut metrics = QueryMetrics::default();
    let mut visiting = HashSet::new();
    for def in &doc.definitions {
        if let Some(selection_set) = operation_selection_set(def) {
            walk_selection_set(selection_set, 1, &fragments, &mut visiting, &mut metrics);
        }
    }
    Ok(metrics)
}

/// Back-compat convenience wrapper around [`analyze_query`] for callers that just want a number
/// and don't need to distinguish "invalid query" from "zero complexity/depth".
pub fn compute_query_depth(query: &str) -> u32 {
    analyze_query(query).map(|m| m.depth).unwrap_or(0)
}

/// See [`compute_query_depth`].
pub fn compute_query_complexity(query: &str) -> u64 {
    analyze_query(query).map(|m| m.complexity).unwrap_or(0)
}

/// Checks that the query depth does not exceed `max_depth`.
/// Returns `Ok(())` if allowed, or an error message string.
pub fn check_query_depth(query: &str, max_depth: u32) -> Result<(), String> {
    let metrics = analyze_query(query).map_err(|e| format!("Invalid GraphQL query: {e}"))?;
    if metrics.depth > max_depth {
        return Err(format!(
            "Query depth limit exceeded: depth {} exceeds maximum {}",
            metrics.depth, max_depth
        ));
    }
    Ok(())
}

/// Checks that the query complexity does not exceed `max_complexity`.
/// Returns `Ok(())` if allowed, or an error message string.
pub fn check_query_complexity(query: &str, max_complexity: u64) -> Result<(), String> {
    let metrics = analyze_query(query).map_err(|e| format!("Invalid GraphQL query: {e}"))?;
    if metrics.complexity > max_complexity {
        return Err(format!(
            "Query complexity limit exceeded: score {} exceeds maximum {}",
            metrics.complexity, max_complexity
        ));
    }
    Ok(())
}

/// Caps a requested limit to MAX_PAGE_SIZE. Use for list/offset pagination.
#[inline]
pub fn cap_page_size(limit: Option<i64>) -> Option<i64> {
    limit.map(|n| n.clamp(1, MAX_PAGE_SIZE))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_simple_query() {
        assert_eq!(analyze_query("{ apiVersion }").unwrap().depth, 1);
    }

    #[test]
    fn depth_nested() {
        assert_eq!(analyze_query("{ a { b { c } } }").unwrap().depth, 3);
    }

    #[test]
    fn depth_ignores_strings() {
        assert_eq!(
            analyze_query(r#"{ a(b: " { { { ") { c } }"#).unwrap().depth,
            2
        );
    }

    #[test]
    fn check_depth_ok() {
        assert!(check_query_depth("{ a { b } }", 3).is_ok());
    }

    #[test]
    fn check_depth_exceeded() {
        let err = check_query_depth("{ a { b { c { d } } } }", 2).unwrap_err();
        assert!(err.contains("exceeds maximum"));
    }

    #[test]
    fn cap_page_size_clamps() {
        assert_eq!(cap_page_size(Some(100)), Some(50));
        assert_eq!(cap_page_size(Some(10)), Some(10));
        assert_eq!(cap_page_size(None), None);
    }

    #[test]
    fn complexity_simple_query() {
        assert_eq!(analyze_query("{ apiVersion }").unwrap().complexity, 1);
    }

    #[test]
    fn complexity_nested() {
        // depth 1 field: +1, depth 2 field: +2, depth 3 field: +3 -> 6
        assert_eq!(analyze_query("{ a { b { c } } }").unwrap().complexity, 6);
    }

    #[test]
    fn complexity_ignores_strings() {
        assert_eq!(
            analyze_query(r#"{ a(b: " { { { ") { c } }"#)
                .unwrap()
                .complexity,
            3
        );
    }

    #[test]
    fn check_complexity_ok() {
        assert!(check_query_complexity("{ a { b } }", 10).is_ok());
    }

    #[test]
    fn check_complexity_exceeded() {
        let err = check_query_complexity("{ a { b { c { d } } } }", 5).unwrap_err();
        assert!(err.contains("complexity"));
        assert!(err.contains("exceeds maximum"));
    }

    /// Aliasing the same field many times at one depth must scale complexity with the field
    /// count, not just the brace nesting — this is the width-blindness the brace-counter had.
    #[test]
    fn complexity_scales_with_aliased_field_width_not_just_depth() {
        let mut query = String::from("{ ");
        for i in 0..500 {
            query.push_str(&format!("a{i}: apiVersion "));
        }
        query.push('}');
        let metrics = analyze_query(&query).unwrap();
        assert_eq!(metrics.depth, 1);
        assert_eq!(metrics.complexity, 500);
        assert!(check_query_complexity(&query, DEFAULT_MAX_QUERY_COMPLEXITY).is_err());
    }

    #[test]
    fn complexity_counts_fields_reached_via_fragment_spread() {
        let query = "{ a { ...Frag } } fragment Frag on T { b c d }";
        let metrics = analyze_query(query).unwrap();
        // a@depth1 (+1) then b,c,d@depth2 via the spread (+2 each) -> 1 + 6 = 7
        assert_eq!(metrics.complexity, 7);
        assert_eq!(metrics.depth, 2);
    }

    #[test]
    fn fragment_cycle_does_not_infinite_loop() {
        let query = "{ ...A } fragment A on T { ...B } fragment B on T { ...A }";
        // Must terminate; exact score isn't the point, absence of a hang/stack overflow is.
        let metrics = analyze_query(query).unwrap();
        assert_eq!(metrics.complexity, 0);
    }

    #[test]
    fn invalid_query_is_reported_not_silently_zero() {
        assert!(analyze_query("{ a ").is_err());
        assert!(check_query_depth("{ a ", 10).is_err());
    }
}
