# Sudatta's Storefront (Next.js)

Minimal luxury storefront for Sudatta's designer sarees. This app replaces the legacy CRA-based `frontend` and runs alongside the Rust GraphQL backend.

## Stack

- **Next.js 16** (App Router) + **TypeScript**
- **Tailwind CSS** (v4) + **shadcn-style UI** (Button, Input, Sheet, Dialog)
- **TanStack Query** + typed **GraphQL client** (guest session / NextAuth)
- **Zod** for schemas and response validation
- **Framer Motion** for hero and micro-interactions
- **lucide-react** for icons

## Getting started

```bash
cp .env.example .env.local
npm install
npm run dev
```

Open [http://localhost:3000](http://localhost:3000). The **admin panel** is at [http://localhost:3000/imtheboss](http://localhost:3000/imtheboss) (Dashboard, Orders, Products, Customers, Settings).

## Scripts

- `npm run dev` - development server
- `npm run build` - production build
- `npm run start` - run production build
- `npm run lint` - ESLint
- `npm run test` - unit/integration tests
- `npm run test:e2e:critical` - critical mocked commerce Playwright journey (provider-safe)
- `npm run test:e2e:journeys` - journey matrix Playwright suite (provider-safe by default)

### E2E safety defaults

- Default CI E2E is mock/offline-safe for external providers.
- Real provider flows (Razorpay/Shiprocket live calls or live webhook side effects) are excluded by default.
- Provider-live runs are opt-in only and require:
  - `RUN_LIVE_PROVIDER_JOURNEYS=1`
  - `PROVIDER_LIVE_TEST_CONFIRM=I_UNDERSTAND_THIS_HITS_REAL_PROVIDERS`

## Environment

See `.env.example`. Key variables:

- `NEXT_PUBLIC_GRAPHQL_URL` - GraphQL endpoint (default: `http://localhost:8080/v2`)
- `NEXT_PUBLIC_SITE_URL` - public site URL used for metadata, sitemap and robots
- `GRAPHQL_URL` - server-only GraphQL endpoint override for Next.js API/auth routes
- `STOREFRONT_ORIGIN` - server-side origin header for session-authenticated GraphQL calls
- `AUTH_SECRET` (or `NEXTAUTH_SECRET`), `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET` - required for Google NextAuth sign-in
- `ADMIN_ALLOWED_EMAILS` - comma-separated admin allowlist for `/imtheboss`

## Learn more

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS](https://tailwindcss.com/docs)
- [TanStack Query](https://tanstack.com/query/latest)
