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
REQUIRE_WEBHOOK_SECRETS=true

SHIPROCKET_EMAIL=api_user@example.com
SHIPROCKET_PASSWORD=your_shiprocket_password
SHIPROCKET_WEBHOOK_SECRET=your_shiprocket_webhook_secret
```

Notes:

- `RAZORPAY_KEY_SECRET` is for Razorpay API auth (server -> Razorpay).
- `RAZORPAY_WEBHOOK_SECRET` is only for webhook signature validation (Razorpay -> server).
- `SHIPROCKET_WEBHOOK_SECRET` is matched against header `x-shiprocket-token`.
- `REQUIRE_WEBHOOK_SECRETS=true` is recommended for staging/production so startup fails fast if either webhook secret is missing.

## 2) Public URL: Caddy reverse proxy (local tunnel now, domain in production)

The site has a single entry point, Caddy (`backend/Caddyfile`, started by `start-services.ps1`).
It sends `/v2`, `/session/*`, `/auth/phone-otp/*`, `/invoices/*` and both webhook paths to
GraphQL, and everything else to the Next.js frontend. So the frontend and the webhooks share one
URL, both locally and on the domain. `/metrics` and `/ready` are not exposed.

### Local (now)

1. `.\start-services.ps1` (starts Caddy on `http://localhost:8000`).
2. Frontend `.env.local`: `NEXT_PUBLIC_GRAPHQL_URL=/v2` and `GRAPHQL_URL=http://127.0.0.1:8080/v2`, then `npm run dev`.
3. Expose Caddy (not `:3000` or `:8080`):

   ```powershell
   cloudflared tunnel --url http://localhost:8000
   ```

4. Cloudflare prints a URL like `https://barbara-deborah-venture-carriers.trycloudflare.com`. Add it to
   backend `ALLOWED_ORIGINS` (and frontend `STOREFRONT_ORIGIN` / `NEXT_PUBLIC_SITE_URL`), then restart
   GraphQL and the frontend.
5. Webhook endpoints:
   - Razorpay: `https://<tunnel-domain>/wheresthemoney/razorpay`
   - Shiprocket: `https://<tunnel-domain>/blastoff/parcelupdate`

Important:

- For `trycloudflare`, you must keep the command running. If it stops, the URL stops working.
- Restarting generates a different URL: update `ALLOWED_ORIGINS` and both dashboards again.
- With `NEXT_PUBLIC_GRAPHQL_URL=/v2`, open the site via Caddy (`http://localhost:8000` or the tunnel),
  not `http://localhost:3000` — the browser's `/v2` calls only reach GraphQL through Caddy.

### Production (VPS, same Caddyfile)

Point the domain's DNS A record at the VPS, open ports 80/443, and set in `backend/.env`:

```env
SITE_ADDRESS=sudattasdesignerboutique.com
CADDY_HTTP_PUBLISH=80:80
CADDY_HTTPS_PUBLISH=443:443
ALLOWED_ORIGINS=https://sudattasdesignerboutique.com
STOREFRONT_URL=https://sudattasdesignerboutique.com
```

Caddy obtains and renews the HTTPS certificate itself. Run the frontend on the VPS on port 3000
(if it runs on the host rather than in Docker, start it with `-H 0.0.0.0` so the Caddy container
can reach it, and keep 3000 closed in the firewall). Webhook endpoints become:

- Razorpay: `https://sudattasdesignerboutique.com/wheresthemoney/razorpay`
- Shiprocket: `https://sudattasdesignerboutique.com/blastoff/parcelupdate`

MySQL, Redis, gRPC and GraphQL are published on `127.0.0.1` only; ntfy (`8090`) is the one other
published port, for the phone app.

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
