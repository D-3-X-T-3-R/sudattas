import { expect, test, type Page, type Route } from "@playwright/test";

type MockLine = {
  cartId: string;
  userId: string;
  variantId: string;
  quantity: string;
  productId: string;
  name: string;
  amountPaise: string;
  formatted: string;
  categoryId: string;
  occasion: string;
  imageUrl: string;
  sizeName: string;
};

type MockOrder = {
  orderId: string;
  totalAmountPaise: string;
  totalAmountFormatted: string;
  statusId: string;
  statusName: string;
};

function productRow(input: {
  id: string;
  name: string;
  pricePaise: number;
  variantId: string;
  sizeName?: string;
  imageUrl?: string;
}) {
  const imageUrl = input.imageUrl ?? "https://example.com/mock-product.jpg";
  return {
    id: input.id,
    name: input.name,
    collection: "Festive",
    price: input.pricePaise / 100,
    pricePaise: input.pricePaise,
    priceFormatted: `Rs ${(input.pricePaise / 100).toFixed(2)}`,
    rating: 4.8,
    reviews: 12,
    fabric: "Silk",
    occasion: "Wedding",
    description: `${input.name} description`,
    image: imageUrl,
    hoverImage: imageUrl,
    images: [imageUrl],
    imageAlt: input.name,
    variantStock: [
      {
        variantId: input.variantId,
        sizeId: "size-1",
        sizeName: input.sizeName ?? "M",
        quantity: 10,
      },
    ],
  };
}

function lineToEnvelopeRow(line: MockLine) {
  return {
    cartId: line.cartId,
    userId: line.userId,
    variantId: line.variantId,
    quantity: line.quantity,
    productDetails: [
      {
        productId: line.productId,
        name: line.name,
        description: `${line.name} description`,
        amountPaise: line.amountPaise,
        formatted: line.formatted,
        categoryId: line.categoryId,
        fabric: "Silk",
        occasion: line.occasion,
        images: [{ url: line.imageUrl, thumbnailUrl: line.imageUrl }],
        variantStock: [
          {
            variantId: line.variantId,
            sizeId: "size-1",
            sizeName: line.sizeName,
            quantity: 10,
          },
        ],
      },
    ],
  };
}

function responseEnvelope(data: unknown, overrides?: { ok?: boolean; message?: string; errorCode?: string | null }) {
  return {
    ok: overrides?.ok ?? true,
    data,
    errorCode: overrides?.errorCode ?? null,
    message: overrides?.message ?? null,
    fieldErrors: null,
    retryable: false,
  };
}

// This helper intentionally keeps the critical mocked commerce flow in one place for the browser journey.
// eslint-disable-next-line max-lines-per-function
async function installCommerceMocks(page: Page) {
  let authenticated = false;
  let mergeCalls = 0;
  let customerCartLoads = 0;
  let verifyCalls = 0;
  let orderListLoads = 0;
  let orderDetailLoads = 0;
  let placeOrderSelections: string[] = [];
  const placeOrderSelectionHistory: string[][] = [];
  let lastShippingSelection: string[] = [];
  let lastOrder: MockOrder | null = null;

  const guestCart: MockLine[] = [];
  const customerCart: MockLine[] = [];

  const products = [
    productRow({ id: "101", name: "Amber Saree", pricePaise: 1_500, variantId: "v-amber", imageUrl: "https://example.com/amber.jpg" }),
    productRow({ id: "202", name: "Emerald Saree", pricePaise: 2_500, variantId: "v-emerald", imageUrl: "https://example.com/emerald.jpg" }),
  ];

  await page.addInitScript(() => {
    class MockRazorpay {
      options: Record<string, unknown>;

      constructor(options: Record<string, unknown>) {
        this.options = options;
      }

      open() {
        const handler = this.options.handler as undefined | ((payload: {
          razorpay_payment_id: string;
          razorpay_order_id: string;
          razorpay_signature: string;
        }) => void);
        handler?.({
          razorpay_payment_id: "pay_mock_1",
          razorpay_order_id: String(this.options.order_id ?? "order_mock_1"),
          razorpay_signature: "sig_mock_1",
        });
      }

      on() {}
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (window as any).Razorpay = MockRazorpay;
  });

  const json = async (route: Route, body: unknown, status = 200) =>
    route.fulfill({ status, contentType: "application/json", body: JSON.stringify(body) });

  await page.route("**/api/auth/session", async (route) => {
    if (!authenticated) {
      await json(route, null);
      return;
    }
    await json(route, {
      expires: "2099-01-01T00:00:00.000Z",
      user: { name: "Mock User", email: "mock@example.com" },
      customerUserId: "42",
      idToken: "header.payload.signature",
    });
  });

  await page.route("**/session/guest", async (route) => {
    await json(route, { session_id: "guest-session-1" });
  });

  await page.route("**/api/products**", async (route) => {
    await json(route, { products, error: null });
  });

  await page.route("**/api/storefront-filters", async (route) => {
    await json(route, {
      categories: [{ categoryId: "cat-1", name: "Festive", thumbnailUrl: "" }],
      occasions: [{ occasionId: "occ-1", occasionName: "Wedding" }],
      moods: [],
      error: null,
    });
  });

  await page.route("**/v2", async (route) => {
    const request = route.request();
    const body = request.postDataJSON() as { query?: string; variables?: Record<string, unknown> };
    const query = body.query ?? "";
    const variables = body.variables ?? {};

    if (query.includes("GetCartItems")) {
      await json(route, {
        data: {
          getCartItems: guestCart.map(lineToEnvelopeRow),
        },
      });
      return;
    }

    if (query.includes("AddCartItem")) {
      const input = variables.input as { variantId: string; quantity: string };
      const product = products.find((row) =>
        row.variantStock?.some((variant) => variant.variantId === input.variantId)
      );
      if (!product) {
        await json(route, { errors: [{ message: "Unknown product" }] }, 400);
        return;
      }
      const existing = guestCart.find((line) => line.variantId === input.variantId);
      if (existing) {
        existing.quantity = String(Number(existing.quantity) + Number(input.quantity));
      } else {
        guestCart.push({
          cartId: String(guestCart.length + 1),
          userId: "0",
          variantId: input.variantId,
          quantity: input.quantity,
          productId: product.id,
          name: product.name,
          amountPaise: String(product.pricePaise),
          formatted: product.priceFormatted ?? "",
          categoryId: "cat-1",
          occasion: "Wedding",
          imageUrl: product.image,
          sizeName: product.variantStock?.[0]?.sizeName ?? "M",
        });
      }
      await json(route, { data: { addCartItem: guestCart.map(lineToEnvelopeRow) } });
      return;
    }

    if (query.includes("UpdateCartItem")) {
      const input = variables.input as { cartId: string; quantity: string };
      const target = guestCart.find((line) => line.cartId === input.cartId);
      if (target) target.quantity = input.quantity;
      await json(route, { data: { updateCartItem: guestCart.map(lineToEnvelopeRow) } });
      return;
    }

    if (query.includes("DeleteCartItem")) {
      const input = variables.input as { cartId: string };
      const next = guestCart.filter((line) => line.cartId !== input.cartId);
      guestCart.splice(0, guestCart.length, ...next);
      await json(route, { data: { deleteCartItem: guestCart.map(lineToEnvelopeRow) } });
      return;
    }

    await route.continue();
  });

  await page.route("**/api/account/cart/merge", async (route) => {
    mergeCalls += 1;
    if (guestCart.length > 0) {
      for (const line of guestCart) {
        const existing = customerCart.find((row) => row.variantId === line.variantId);
        if (existing) {
          existing.quantity = String(Number(existing.quantity) + Number(line.quantity));
        } else {
          customerCart.push({
            ...line,
            cartId: String(100 + customerCart.length + 1),
            userId: "42",
          });
        }
      }
      guestCart.splice(0, guestCart.length);
    }
    await json(route, responseEnvelope({ merged: customerCart.length, deletedGuestItems: 2 }));
  });

  await page.route("**/api/account/cart", async (route) => {
    customerCartLoads += 1;
    await json(route, responseEnvelope(customerCart.map(lineToEnvelopeRow)));
  });

  await page.route("**/api/account/addresses", async (route) => {
    await json(route, responseEnvelope([
      {
        shippingAddressId: "addr-1",
        recipientName: "Mock User",
        phoneNumber: "+919999999999",
        isDefault: true,
        country: "India",
        stateRegion: "KA",
        city: "Bengaluru",
        postalCode: "560001",
        road: "MG Road",
        apartmentNoOrName: "Flat 2A",
      },
    ]));
  });

  await page.route("**/api/checkout/shipping-estimate", async (route) => {
    const body = route.request().postDataJSON() as { selectedCartLineIds?: string[] };
    lastShippingSelection = body.selectedCartLineIds ?? [];
    const subtotal = customerCart
      .filter((line) => lastShippingSelection.includes(line.cartId))
      .reduce((sum, line) => sum + Number(line.amountPaise) * Number(line.quantity), 0);
    await json(route, responseEnvelope({
      shippingAmountPaise: "500",
      courierName: "Shiprocket",
      estimatedDeliveryDays: 3,
      itemSubtotalPaise: String(subtotal),
      orderTotalPaise: String(subtotal + 500),
      quoteAvailable: true,
      note: null,
    }));
  });

  await page.route("**/api/checkout/place-order", async (route) => {
    const body = route.request().postDataJSON() as { selectedCartLineIds?: string[] };
    placeOrderSelections = body.selectedCartLineIds ?? [];
    placeOrderSelectionHistory.push([...placeOrderSelections]);
    const selectedLines = customerCart.filter((line) => placeOrderSelections.includes(line.cartId));
    const subtotal = selectedLines.reduce((sum, line) => sum + Number(line.amountPaise) * Number(line.quantity), 0);
    const total = subtotal + 500;

    customerCart.splice(
      0,
      customerCart.length,
      ...customerCart.filter((line) => !placeOrderSelections.includes(line.cartId))
    );

    lastOrder = {
      orderId: "9001",
      totalAmountPaise: String(total),
      totalAmountFormatted: `Rs ${(total / 100).toFixed(2)}`,
      statusId: "2",
      statusName: "confirmed",
    };

    await json(route, responseEnvelope({
      order: {
        orderId: lastOrder.orderId,
        totalAmountPaise: lastOrder.totalAmountPaise,
        totalAmountFormatted: lastOrder.totalAmountFormatted,
        statusId: lastOrder.statusId,
      },
      paymentIntent: {
        intentId: "pi-1",
        razorpayOrderId: "order_mock_1",
        razorpayKeyId: "key_mock_1",
        orderId: lastOrder.orderId,
        amountPaise: lastOrder.totalAmountPaise,
        currency: "INR",
        status: "pending",
      },
      idempotency: {
        placeOrderKey: "place-key-1",
        verifyKey: "verify-key-1",
      },
    }));
  });

  await page.route("**/api/checkout/verify-payment", async (route) => {
    verifyCalls += 1;
    await json(route, responseEnvelope({
      verified: true,
      paymentState: "paid",
      orderStatusId: "2",
      orderUiState: "processing",
      verifyKey: "verify-key-1",
    }));
  });

  await page.route("**/api/account/profile", async (route) => {
    await json(route, responseEnvelope({
      userId: "42",
      email: "mock@example.com",
      fullName: "Mock User",
      createDate: "2026-01-01T00:00:00.000Z",
    }));
  });

  await page.route("**/api/account/orders", async (route) => {
    orderListLoads += 1;
    await json(
      route,
      responseEnvelope(
        lastOrder
          ? [
              {
                orderId: lastOrder.orderId,
                userId: "42",
                orderDate: "2026-01-01T00:00:00.000Z",
                totalAmountPaise: lastOrder.totalAmountPaise,
                totalAmountFormatted: lastOrder.totalAmountFormatted,
                statusId: lastOrder.statusId,
                statusName: "confirmed",
              },
            ]
          : []
      )
    );
  });

  await page.route("**/api/account/orders/9001", async (route) => {
    orderDetailLoads += 1;
    await json(route, responseEnvelope({
      order: {
        orderId: "9001",
        userId: "42",
        orderDate: "2026-01-01T00:00:00.000Z",
        totalAmountPaise: lastOrder?.totalAmountPaise ?? "0",
        totalAmountFormatted: lastOrder?.totalAmountFormatted ?? "Rs 0.00",
        statusId: "2",
        statusName: "confirmed",
        orderDetails: [
          {
            orderDetailId: "od-1",
            variantId: "v-amber",
            quantity: "1",
            pricePaise: "1500",
            priceFormatted: "Rs 15.00",
            productDetails: [{ productId: "101", name: "Amber Saree", formatted: "Rs 15.00", images: [{ url: "https://example.com/amber.jpg" }] }],
          },
        ],
      },
      statusName: "confirmed",
      paymentIntents: [
        {
          intentId: "pi-1",
          amountPaise: lastOrder?.totalAmountPaise ?? "0",
          currency: "INR",
          status: "paid",
          razorpayPaymentId: "pay_mock_1",
          createdAt: "2026-01-01T00:00:00.000Z",
        },
      ],
      shipments: [
        {
          shipmentId: "ship-1",
          status: "in_transit",
          carrier: "Shiprocket",
          awbCode: "AWB-123",
          createdAt: "2026-01-01T00:00:00.000Z",
          trackingEventsJson: JSON.stringify([{ label: "Picked up", at: "2026-01-01T00:00:00.000Z" }]),
          shiprocketStatusId: "18",
          shiprocketStatusLabel: "In Transit",
        },
      ],
      events: [
        {
          eventId: "evt-1",
          eventType: "payment_client_verified",
          fromStatus: "pending",
          toStatus: "confirmed",
          actorType: "customer",
          message: "Payment verified",
          createdAt: "2026-01-01T00:00:00.000Z",
        },
      ],
      fulfillmentState: "processing",
      paymentState: "paid",
    }));
  });

  return {
    setAuthenticated(value: boolean) {
      authenticated = value;
    },
    get mergeCalls() {
      return mergeCalls;
    },
    get customerCartLoads() {
      return customerCartLoads;
    },
    get orderDetailLoads() {
      return orderDetailLoads;
    },
    get orderListLoads() {
      return orderListLoads;
    },
    get verifyCalls() {
      return verifyCalls;
    },
    get placeOrderSelections() {
      return placeOrderSelections;
    },
    get placeOrderSelectionHistory() {
      return placeOrderSelectionHistory;
    },
    get lastShippingSelection() {
      return lastShippingSelection;
    },
    get remainingCustomerCart() {
      return customerCart.map((line) => line.name);
    },
    get customerCartLines() {
      return customerCart.map((line) => ({ cartId: line.cartId, name: line.name }));
    },
  };
}

async function addGuestLine(page: Page, variantId: string, quantity: string) {
  await page.evaluate(
    async ({ variantId: inputVariantId, quantity: inputQuantity }) => {
      const response = await fetch("/v2", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          query: `
            mutation AddCartItem($input: NewCart!) {
              addCartItem(item: $input) { cartId }
            }
          `,
          variables: {
            input: {
              variantId: inputVariantId,
              quantity: inputQuantity,
            },
          },
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to seed guest bag: ${response.status}`);
      }
    },
    { variantId, quantity }
  );
}

test("critical commerce browser journey", async ({ page }) => {
  const mocks = await installCommerceMocks(page);

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await expect(page.getByRole("button", { name: /sign in/i })).toBeVisible();
  await expect(page.locator('button[aria-label^="Quick view "]').first()).toBeVisible();

  await expect.poll(async () =>
    page.evaluate(() => window.localStorage.getItem("sudattas_guest_session"))
  ).toBe("guest-session-1");

  await addGuestLine(page, "v-amber", "1");
  await addGuestLine(page, "v-emerald", "1");

  await page.goto("/bag", { waitUntil: "domcontentloaded" });
  await expect(page.locator("body")).toContainText("Amber Saree");
  await expect(page.locator("body")).toContainText("Emerald Saree");

  await page.goto("/", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: /sign in/i }).click();
  await expect(page.locator("body")).toContainText("Continue with Google or sign in with phone OTP.");

  mocks.setAuthenticated(true);
  await page.goto("/bag", { waitUntil: "domcontentloaded" });

  await expect.poll(() => mocks.mergeCalls).toBe(1);
  await expect(page.locator("body")).toContainText("Amber Saree");
  await expect(page.locator("body")).toContainText("Emerald Saree");

  const amberCartId = mocks.customerCartLines.find((line) => line.name === "Amber Saree")?.cartId;
  expect(amberCartId).toBeTruthy();

  const checkoutFlow = await page.evaluate(
    async ({ selectedCartLineIds }) => {
      const shippingResponse = await fetch("/api/checkout/shipping-estimate", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ shippingAddressId: "addr-1", selectedCartLineIds }),
      });
      const shipping = await shippingResponse.json();

      const placeOrderResponse = await fetch("/api/checkout/place-order", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ shippingAddressId: "addr-1", selectedCartLineIds, idempotencyKey: "place-key-1" }),
      });
      const placeOrder = await placeOrderResponse.json();

      const verifyResponse = await fetch("/api/checkout/verify-payment", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          orderId: placeOrder.data.order.orderId,
          razorpayPaymentId: "pay_mock_1",
          razorpayOrderId: placeOrder.data.paymentIntent.razorpayOrderId,
          razorpaySignature: "sig_mock_1",
          idempotencyKey: "verify-key-1",
        }),
      });
      const verify = await verifyResponse.json();

      return { shipping, placeOrder, verify };
    },
    { selectedCartLineIds: [amberCartId] }
  );

  expect(checkoutFlow.shipping.ok).toBeTruthy();
  expect(checkoutFlow.shipping.data.itemSubtotalPaise).toBe("1500");
  expect(checkoutFlow.shipping.data.orderTotalPaise).toBe("2000");
  expect(checkoutFlow.placeOrder.ok).toBeTruthy();
  expect(checkoutFlow.verify.ok).toBeTruthy();

  await page.goto("/checkout/success?orderId=9001", { waitUntil: "domcontentloaded" });
  await expect(page).toHaveURL(/\/checkout\/success\?orderId=9001/);

  expect(mocks.placeOrderSelectionHistory.some((selection) => selection.length === 1)).toBeTruthy();
  expect(mocks.placeOrderSelectionHistory.some((selection) => selection.length > 1)).toBeFalsy();
  expect(mocks.verifyCalls).toBe(1);

  await page.goto("/bag", { waitUntil: "domcontentloaded" });
  await expect(page.locator("body")).toContainText("Emerald Saree");
  await expect(page.locator("body")).not.toContainText("Amber Saree");
  expect(mocks.remainingCustomerCart).toEqual(["Emerald Saree"]);

  await page.goto("/profile", { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: "Orders" }).click();
  await expect(page.locator("body")).toContainText("Order #9001");
  await expect.poll(() => mocks.orderListLoads).toBeGreaterThan(0);
  await page.getByRole("button", { name: /Refresh tracking/i }).click();
  await expect(page.locator("body")).toContainText("AWB AWB-123");
  await expect.poll(() => mocks.orderDetailLoads).toBeGreaterThan(0);
});
