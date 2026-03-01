# Razorpay integration

After your Razorpay account is created/verified, wire it up as below.

## 1. Get credentials from Razorpay Dashboard

1. Log in at [dashboard.razorpay.com](https://dashboard.razorpay.com).
2. **API Keys** (Settings → API Keys):
   - **Key ID** (e.g. `rzp_test_...` or `rzp_live_...`)
   - **Key Secret** (generate/show once)
3. **Webhooks** (Settings → Webhooks):
   - Add endpoint: `https://your-graphql-host/webhook/razorpay`
   - Subscribe to: `payment.captured`, `payment.failed` (and any others you need)
   - Copy the **Signing secret** (used to verify webhook payloads)

## 2. Backend environment variables

Set these where your **core_operations** and **graphql** services run (e.g. `.env` or your deployment env):

| Variable | Required | Description |
|----------|----------|-------------|
| `RAZORPAY_KEY_ID` | Yes, for creating orders | API Key ID (safe to expose to frontend; we only send it in API responses). |
| `RAZORPAY_KEY_SECRET` | Yes, for orders + verification | API Key Secret. **Never** expose; used for Orders API and `verifyRazorpayPayment` signature check. |
| `RAZORPAY_WEBHOOK_SECRET` | Recommended in production | Webhook signing secret. If set, `POST /webhook/razorpay` **requires** a valid `x-razorpay-signature`. |

- Without `RAZORPAY_KEY_ID` / `RAZORPAY_KEY_SECRET`, the backend will **not** call Razorpay’s Orders API; `createPaymentIntent` (when backend creates the order) will fail.
- Without `RAZORPAY_WEBHOOK_SECRET`, webhooks are accepted without signature verification (only acceptable for local/dev).

See `backend/.env.example` for a template.

## 3. Backend flow (already implemented)

- **Checkout / place order**  
  Your app calls `place_order` (or equivalent). The backend creates a payment intent and, when `razorpay_order_id` is not provided, calls Razorpay `orders.create` with the order total (paise), then returns `razorpay_key_id`, `razorpay_order_id`, amount, currency, etc.

- **Frontend**  
  Use the returned `razorpay_key_id` and `razorpay_order_id` with [Razorpay Checkout](https://razorpay.com/docs/payments/payment-gateway/web-integration/standard/). Load the Checkout script, open the modal with `order_id` and `key_id`; on success you get `razorpay_payment_id`, `razorpay_order_id`, and `razorpay_signature`.

- **Verify payment (client return)**  
  After Checkout succeeds, call the GraphQL mutation `verifyRazorpayPayment` with:
  - `orderId` (your order id)
  - `razorpayPaymentId`
  - `razorpayOrderId`
  - `razorpaySignature`  

  The backend verifies the signature with `RAZORPAY_KEY_SECRET` and, if valid, marks the payment intent as `ClientVerified` and stores `razorpay_payment_id`.

- **Webhook (source of truth)**  
  Razorpay sends `payment.captured` (and `payment.failed`) to your `POST /webhook/razorpay` URL. The graphql service verifies `x-razorpay-signature` when `RAZORPAY_WEBHOOK_SECRET` is set, then forwards to core; the handler updates the payment intent and order (e.g. marks paid). Webhooks are the authority for final payment status.

## 4. Frontend checklist

1. **Create payment intent (server-authoritative)**  
   Don’t create the Razorpay order on the client. After place-order, get the payment intent from your API (e.g. `createPaymentIntent` with only `orderId`, `userId`, `amountPaise`, `currency`; leave `razorpayOrderId` unset so the backend creates the Razorpay order).

2. **Open Razorpay Checkout**  
   Use the response:
   - `razorpay_key_id` → Checkout key
   - `razorpay_order_id` → Razorpay order id  
   Amount/currency are already set on the order; don’t override them on the client.

3. **On success handler**  
   You receive `razorpay_payment_id`, `razorpay_order_id`, `razorpay_signature`. Call `verifyRazorpayPayment(orderId, razorpayPaymentId, razorpayOrderId, razorpaySignature)` so the backend can verify the signature and set the intent to `ClientVerified`.

4. **UI state**  
   Treat “paid” or “order confirmed” only after your backend says so (e.g. order status from your API or webhook-driven update), not only from the Checkout success callback.

## 5. Test vs live

- **Test mode:** Use **Test** API keys (`rzp_test_...`) and Test mode in the Dashboard. No real money.
- **Live mode:** Use **Live** keys (`rzp_live_...`), complete KYC if required, and set the webhook URL to your production GraphQL base URL.

## 6. Optional: existing DB migration

If your database was created before the Razorpay payment-intent status values were added, ensure the `payment_intents.status` enum includes `client_verified`:

```sql
ALTER TABLE payment_intents
MODIFY COLUMN status ENUM('pending', 'processed', 'failed', 'needs_review', 'client_verified') NOT NULL DEFAULT 'pending';
```

New installs using `database/sql_dump/01_schema.sql` already have this.
