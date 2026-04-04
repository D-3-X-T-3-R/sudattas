/**
 * Shared GraphQL client entry point.
 * - gqlAdmin: browser admin client via server-only /api/admin/graphql proxy.
 * - gqlWithSession: for storefront (guest session only).
 */

export { gqlAdmin } from "./graphqlAdmin";
export { gqlWithSession } from "./graphqlWithSession";
