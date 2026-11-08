/**
 * Shared shape for GraphqlRequestError, kept as a type-only interface (no runtime code) rather
 * than a shared base class because graphqlClient.ts (browser) and graphqlWithSession.ts (server)
 * each carry their own module boundary: graphqlClient.ts pulls in browser-only modules
 * (authStore, session) that graphqlWithSession.ts must not import. A type-only import of this
 * interface is safe for both.
 */
export interface GraphqlErrorLike {
  code?: string;
  grpcCode?: number;
}
