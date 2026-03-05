import "next-auth";

declare module "next-auth" {
  interface Session {
    accessToken?: string;
    /** Google id_token (JWT). Send this as Bearer so the backend can validate it. */
    idToken?: string;
  }
}
