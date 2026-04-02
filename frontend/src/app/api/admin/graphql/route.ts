import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

export async function POST(request: Request) {
  return forwardAdminGraphql(request);
}
