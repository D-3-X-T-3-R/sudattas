import { forwardAdminGraphql } from "@/lib/admin-graphql-server";

const SHIPMENT_ROOTS = [
  "getShipment",
  "createShipment",
  "updateShipment",
  "adminMarkOrderShipped",
  "adminMarkOrderDelivered",
  "syncOrderShipmentsFromShiprocket",
];

export async function POST(request: Request) {
  return forwardAdminGraphql(request, { allowedRoots: SHIPMENT_ROOTS });
}

