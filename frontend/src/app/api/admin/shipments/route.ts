import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const SHIPMENT_ROOTS = [
  "searchShipment",
  "createShipment",
  "updateShipment",
  "markOrderShipped",
  "markOrderDelivered",
  "updateOrderStatus",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: SHIPMENT_ROOTS });
}

