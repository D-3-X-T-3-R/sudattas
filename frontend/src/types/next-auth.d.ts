import "next-auth";
import "next-auth/jwt";

declare module "next-auth" {
  interface Session {
    accessToken?: string;
    /** Google id_token (JWT). Send this as Bearer so the backend can validate it. */
    idToken?: string;
    customerUserId?: string;
    isAdmin?: boolean;
  }
}

declare module "next-auth/jwt" {
  interface JWT {
    accessToken?: string;
    idToken?: string;
    customerUserId?: string;
    isAdmin?: boolean;
  }
}
