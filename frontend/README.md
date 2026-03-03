# Sudatta's Storefront (Next.js)

Minimal luxury storefront for Sudatta's designer sarees. This app replaces the legacy CRA-based `frontend` and runs alongside the Rust GraphQL backend.

## Stack

- **Next.js 16** (App Router) + **TypeScript**
- **Tailwind CSS** (v4) + **shadcn-style UI** (Button, Input, Sheet, Dialog)
- **TanStack Query** + typed **GraphQL client** (guest session / Auth0)
- **Zod** for schemas and response validation
- **Framer Motion** for hero and micro-interactions
- **lucide-react** for icons

## Getting started

```bash
cp .env.example .env.local   # optional: set NEXT_PUBLIC_GRAPHQL_URL if backend is not on localhost:8080
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000). The **admin panel** is at [http://localhost:3000/imtheboss](http://localhost:3000/imtheboss) (Dashboard, Orders, Products, Customers, Settings). Ensure the GraphQL backend is running (e.g. `http://localhost:8080/v2`) and that guest session is enabled (Redis) if you use cart/checkout. For admin mutations, set `NEXT_PUBLIC_ADMIN_API_KEY` to match the backend `ADMIN_API_KEY`.

## Scripts

- `npm run dev` — development server
- `npm run build` — production build
- `npm run start` — run production build
- `npm run lint` — ESLint

## Environment

See `.env.example`. Key variables:

- `NEXT_PUBLIC_GRAPHQL_URL` — GraphQL endpoint (default: `http://localhost:8080/v2`)
- `NEXT_PUBLIC_GRAPHQL_TOKEN` / `NEXT_PUBLIC_GRAPHQL_SESSION_ID` — optional fixed auth for dev
- `NEXT_PUBLIC_ADMIN_API_KEY` — for admin panel GraphQL calls (must match backend `ADMIN_API_KEY`)
- `NEXT_PUBLIC_AUTH0_DOMAIN` / `NEXT_PUBLIC_AUTH0_CLIENT_ID` — optional Auth0; wire `AuthSync` and login UI when used

## Cutover from CRA `frontend`

This app is the canonical storefront. To fully switch:

1. Point your host or reverse proxy to this app’s build output (e.g. `frontend/out` for static export or the Node server for `next start`).
2. Use `NEXT_PUBLIC_*` env vars instead of `REACT_APP_*`.
3. The legacy `frontend` (CRA) can be deprecated or kept for reference.

## Learn more

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [TanStack Query](https://tanstack.com/query/latest)
