import { gqlAdmin } from "./graphql-client";

/** One line item on an admin order detail view. */
export interface AdminOrderDetailLine {
  orderDetailId: string;
  variantId: string;
  quantity: string;
  priceFormatted: string;
  productName: string | null;
  productId: string | null;
}

/** Full order + lines for admin detail page. */
export interface AdminOrderDetail {
  orderId: string;
  userId: string;
  orderDate: string;
  shippingAddressId: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  lines: AdminOrderDetailLine[];
}

const ADMIN_ORDER_DETAIL_QUERY = `query AdminOrderDetail($search: SearchOrder!) {
  searchOrder(search: $search) {
    orderId
    userId
    orderDate
    shippingAddressId
    totalAmountPaise
    totalAmountFormatted
    statusId
    orderDetails {
      orderDetailId
      variantId
      quantity
      priceFormatted
      productDetails {
        productId
        name
      }
    }
  }
}`;

/** Load one order by id with line items (admin). Returns null if not found. */
export async function fetchAdminOrderById(orderId: string): Promise<AdminOrderDetail | null> {
  const id = orderId.trim();
  if (!id) return null;
  const data = await gqlAdmin<{
    searchOrder?: Array<{
      orderId: string;
      userId: string;
      orderDate: string;
      shippingAddressId: string;
      totalAmountPaise: string;
      totalAmountFormatted: string;
      statusId: string;
      orderDetails: Array<{
        orderDetailId: string;
        variantId: string;
        quantity: string;
        priceFormatted: string;
        productDetails: Array<{ productId: string; name: string }>;
      }>;
    }>;
  }>(ADMIN_ORDER_DETAIL_QUERY, {
    search: { userId: "", orderId: id, limit: "1" },
  });
  const row = data?.searchOrder?.[0];
  if (!row) return null;
  const lines: AdminOrderDetailLine[] = (row.orderDetails ?? []).map((d) => {
    const p = d.productDetails?.[0];
    return {
      orderDetailId: d.orderDetailId,
      variantId: d.variantId,
      quantity: d.quantity,
      priceFormatted: d.priceFormatted,
      productName: p?.name ?? null,
      productId: p?.productId ?? null,
    };
  });
  return {
    orderId: row.orderId,
    userId: row.userId,
    orderDate: row.orderDate,
    shippingAddressId: row.shippingAddressId,
    totalAmountPaise: String(row.totalAmountPaise ?? ""),
    totalAmountFormatted: row.totalAmountFormatted,
    statusId: row.statusId,
    lines,
  };
}

const UPDATE_ORDER_MUTATION = `mutation UpdateAdminOrder($order: OrderMutation!) {
  updateOrder(order: $order) {
    orderId
    statusId
  }
}`;

/**
 * Admin: change order status via updateOrder. Backend enforces allowed state transitions.
 * Other fields are copied from the loaded order snapshot.
 */
export async function updateAdminOrderStatus(
  order: AdminOrderDetail,
  newStatusId: string
): Promise<void> {
  await gqlAdmin(UPDATE_ORDER_MUTATION, {
    order: {
      orderId: order.orderId,
      userId: order.userId,
      orderDate: order.orderDate || new Date().toISOString(),
      shippingAddressId: order.shippingAddressId,
      totalAmountPaise: String(order.totalAmountPaise ?? ""),
      statusId: newStatusId.trim(),
    },
  });
}
