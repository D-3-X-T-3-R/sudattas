# Webhook Setup (Razorpay + Shiprocket)

This project uses custom webhook paths:

- Razorpay: `POST /wheresthemoney/razorpay`
- Shiprocket: `POST /blastoff/parcelupdate`

## 1) Required backend env vars

Set these in backend runtime env:

```env
RAZORPAY_KEY_ID=rzp_test_xxxxxxxx
RAZORPAY_KEY_SECRET=your_razorpay_key_secret
RAZORPAY_WEBHOOK_SECRET=your_razorpay_webhook_secret

SHIPROCKET_EMAIL=api_user@example.com
SHIPROCKET_PASSWORD=your_shiprocket_password
SHIPROCKET_WEBHOOK_SECRET=your_shiprocket_webhook_secret
```

Notes:

- `RAZORPAY_KEY_SECRET` is for Razorpay API auth (server -> Razorpay).
- `RAZORPAY_WEBHOOK_SECRET` is only for webhook signature validation (Razorpay -> server).
- `SHIPROCKET_WEBHOOK_SECRET` is matched against header `x-shiprocket-token`.

## 2) Cloudflare tunnel URL for localhost

Run backend GraphQL locally (typically on `http://localhost:8080`), then expose it:

```powershell
cloudflared tunnel --url http://localhost:8080
```

Cloudflare prints a public URL like:

`https://barbara-deborah-venture-carriers.trycloudflare.com`

Use it to build webhook endpoints:

- Razorpay: `https://<tunnel-domain>/wheresthemoney/razorpay`
- Shiprocket: `https://<tunnel-domain>/blastoff/parcelupdate`

Important:

- For `trycloudflare`, you must keep the command running.
- If the process stops, webhook URL stops working.
- Restarting may generate a different URL.

## 3) Razorpay dashboard configuration

1. Dashboard -> Developers -> Webhooks
2. Add/Edit endpoint URL:
   - `https://<tunnel-domain>/wheresthemoney/razorpay`
3. Set webhook secret in Razorpay.
4. Set the same value in backend env `RAZORPAY_WEBHOOK_SECRET`.

## 4) Shiprocket dashboard configuration

1. Shiprocket -> Settings -> Webhooks (or API Webhook section)
2. Set endpoint URL:
   - `https://<tunnel-domain>/blastoff/parcelupdate`
3. Set token/header:
   - `x-shiprocket-token: <SHIPROCKET_WEBHOOK_SECRET>`

## 5) Validate quickly

1. Restart backend after env updates.
2. Send webhook test events from both dashboards.
3. Confirm no `401`/signature mismatch in backend logs.

