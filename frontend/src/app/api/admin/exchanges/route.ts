import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const EXCHANGES_ROOTS = [
  "searchExchangeRequests",
  "adminMarkExchangeReceived",
  "adminUpdateExchangeStatus",
  "scheduleExchangePickup",
  "syncExchangePickup",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: EXCHANGES_ROOTS });
}
