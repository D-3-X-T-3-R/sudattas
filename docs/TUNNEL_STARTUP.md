# Starting the system on a new tunnel

Runbook for coming back to the project and running it on a fresh `trycloudflare` URL. Every
quick-tunnel restart gives a **new URL**, and that URL has to be updated in a few places. This
covers all of them in order.

How it fits together (details in [WEBHOOK_SETUP.md](WEBHOOK_SETUP.md)):

```
tunnel URL ──> Caddy :8000 ──┬─ /v2, /session, /auth/phone-otp, /invoices, webhooks ─> GraphQL :8080 (Docker)
                             └─ everything else ─────────────────────────────────────> Next.js :3000 (npm run dev)
```

## 0. Before you start

- Docker Desktop is running.
- `cloudflared` is installed (`cloudflared --version`).

## 1. Save the latest database first

The MySQL container has no data volume: `start-services.ps1` backs it up, recreates it and restores the
backup. The backup can only run while MySQL is **running**. After a reboot the container is stopped,
the backup is skipped, and the script restores an **older** backup, so you'd lose the last session's data.

Start the old container once so the script can back it up:

```powershell
docker start sudattas-mysql
```

Ignore "No such container" — that just means there's nothing to save.

## 2. Start the tunnel and copy the new URL

Start the tunnel first, so you only restart the services once with the right URL. Run it in its own
window and **keep it open** (closing it kills the URL):

```powershell
cloudflared tunnel --url http://localhost:8000
```

Copy the `https://<words>.trycloudflare.com` URL it prints. Below, it's written as `<TUNNEL>`.
It will show errors (502) until the services are up. That's expected.

## 3. Update the URL everywhere

Replace the **old** tunnel URL with `<TUNNEL>`. Don't leave old ones behind.

| Where | Setting | Value |
|---|---|---|
| `backend/.env` | `ALLOWED_ORIGINS` | `http://127.0.0.1:3000,http://localhost:3000,http://localhost:8000,<TUNNEL>` |
| `backend/.env` | `STOREFRONT_URL` | `<TUNNEL>` (links in admin alerts and customer emails; use the domain once live) |
| `frontend/.env` | `NEXTAUTH_URL` | `<TUNNEL>` |
| `frontend/.env` | `NEXT_PUBLIC_SITE_URL` | `<TUNNEL>` |
| [Google Cloud console](https://console.cloud.google.com/apis/credentials) → OAuth client `300473955341-3eqkdvlm…` | Authorized JavaScript origins | `<TUNNEL>` |
| same client | Authorized redirect URIs | `<TUNNEL>/api/auth/callback/google` |
| Razorpay Dashboard → Developers → Webhooks | Endpoint URL | `<TUNNEL>/wheresthemoney/razorpay` |
| Shiprocket → Settings → Webhooks | Endpoint URL | `<TUNNEL>/blastoff/parcelupdate` (token header unchanged) |

Leave these alone, they don't depend on the tunnel:

- `frontend/.env`: `NEXT_PUBLIC_GRAPHQL_URL=/v2`, `GRAPHQL_URL=http://127.0.0.1:8080/v2`,
  `STOREFRONT_ORIGIN=http://localhost:3000`
- Webhook secrets in both dashboards and `backend/.env`

Google can take a few minutes to apply the new URIs.

### ntfy (phone alerts): only if your PC's IP changed

```powershell
Get-NetIPAddress -AddressFamily IPv4 -InterfaceAlias Wi-Fi | Select-Object IPAddress
```

If it's no longer the IP in `NTFY_BASE_URL` (`backend/.env`), update it there and in the ntfy app's server
setting (`http://<IP>:8090`). A DHCP reservation on the router keeps it fixed.

## 4. Start the backend

```powershell
cd D:\personal\sudattas\backend
.\start-services.ps1
```

This backs up and restores the DB, applies migrations, rebuilds and starts everything: MySQL, Redis,
ntfy, Core Operations, GraphQL and Caddy. The Rust build takes a few minutes.

## 5. Start the frontend

In another window:

```powershell
cd D:\personal\sudattas\frontend
npm run dev
```

Restart it any time you change `frontend/.env`, because `NEXT_PUBLIC_*` and `NEXTAUTH_URL` are only
read at startup.

## 6. Check it works

| Check | Expected |
|---|---|
| Open `<TUNNEL>` | storefront loads |
| Add an item to the bag | bag updates (browser reaches GraphQL through Caddy) |
| Sign in with Google | returns to `<TUNNEL>`, signed in |
| Phone browser: `http://<IP>:8090/v1/health` | `{"healthy":true}` |
| Place a COD test order | "New order" push on the phone + admin email |
| Razorpay / Shiprocket dashboard "send test webhook" | no `401` in GraphQL logs |

Useful logs:

```powershell
docker logs -f sudattas-graphql           # webhooks, CSRF rejections
docker logs -f sudattas-core-operations   # "admin alert pushed via ntfy", email sends
docker logs -f sudattas-caddy             # routing / upstream errors
```

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| Tunnel shows 502 | Caddy, or `npm run dev`, isn't running yet |
| Page loads but bag/wishlist don't work | Site opened on `:3000` instead of the tunnel or `:8000`, or `NEXT_PUBLIC_GRAPHQL_URL` isn't `/v2` (restart `npm run dev` after changing it) |
| Logged-in actions fail; GraphQL log says `CSRF: session auth rejected` | `<TUNNEL>` missing from `ALLOWED_ORIGINS`: fix it, then `docker compose up -d --no-deps graphql` |
| Google `Error 400: redirect_uri_mismatch` | Redirect URI not added or not applied yet. Must be exactly `<TUNNEL>/api/auth/callback/google` |
| Sign-in bounces to `localhost:3000` | `NEXTAUTH_URL` still old: update it and restart `npm run dev` |
| No push on phone | ntfy stats log shows `subscribers=0`: the app isn't connected. Check the IP, the firewall rule (Wi-Fi must be a **Private** network), and that the app subscribes to `NTFY_TOPIC` on `http://<IP>:8090`, not on ntfy.sh |
| Push rejected (`ntfy error HTTP 403`) in core_operations log | `NTFY_TOPIC` isn't a topic the `backend` user can write to (`docker exec sudattas-ntfy ntfy access`) |
| Webhook `401` | Dashboard secret or Shiprocket token doesn't match `backend/.env` |

Changed only `backend/.env`? You don't need the full script:
`docker compose up -d --no-deps graphql core_operations` (from `backend/`).

## When you're done

Back up the database **while it's still running**, then stop. Don't run `docker compose down` without a
backup — it deletes the MySQL container and its data:

```powershell
cd D:\personal\sudattas\backend
.\backup-db.ps1
docker compose stop
```

Then stop `npm run dev` and the tunnel with Ctrl+C. The next time, start again from step 1.
