/**
 * Shared GraphQL client entry point.
 * - gqlAdmin: for admin dashboard (Bearer / admin key).
 * - gqlWithSession: for storefront (guest session only).
 */

export { gqlAdmin } from "./graphqlAdmin";
export { gqlWithSession } from "./graphqlWithSession";
