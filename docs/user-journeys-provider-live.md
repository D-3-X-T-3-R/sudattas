# Provider-Live User Journeys (Opt-In Only)

Last Updated: 2026-05-07  
Status: **Not part of default CI**

## Purpose

This file contains journeys that can hit real external providers (Razorpay, Shiprocket, live webhooks, and provider side effects).

Default CI must stay deterministic and provider-safe, so these journeys are excluded unless explicitly enabled.

## Required Opt-In Gates

All live-provider runs must require:

- `RUN_LIVE_PROVIDER_JOURNEYS=1`
- `PROVIDER_LIVE_TEST_CONFIRM=I_UNDERSTAND_THIS_HITS_REAL_PROVIDERS`

Logistics-specific runs should also require:

- `RUN_LIVE_LOGISTICS_TESTS=1`

## Provider-Live Journey Set

| ID | Source Mapping | Tags | Live Provider Journey | Notes |
|---|---|---|---|---|
| UJPL-001 | UJ-088 | `@provider-live @razorpay-live` | Razorpay live webhook arrives before client verification. | Real webhook delivery/ordering test. |
| UJPL-002 | UJ-089 | `@provider-live @razorpay-live` | Razorpay live webhook replay duplicate handling. | Real duplicate webhook replay test. |
| UJPL-003 | UJ-090 | `@provider-live @razorpay-live` | Razorpay live webhook amount mismatch path. | Requires signed real provider payload. |
| UJPL-004 | UJ-091 | `@provider-live @razorpay-live` | Razorpay live webhook currency mismatch path. | Requires signed real provider payload. |
| UJPL-005 | (Provider-only) | `@provider-live @razorpay-live` | Razorpay live order creation + capture callback reconciliation. | Real provider-side payment lifecycle. |
| UJPL-006 | (Provider-only) | `@provider-live @razorpay-live` | Razorpay live refund webhook reconciliation. | Real refund side effects. |
| UJPL-007 | (Provider-only) | `@provider-live @shiprocket-live` | Shiprocket live booking from eligible shipped order. | Requires live Shiprocket credentials. |
| UJPL-008 | (Provider-only) | `@provider-live @shiprocket-live` | Shiprocket live cancellation after booking. | Real provider-side cancellation state. |
| UJPL-009 | (Provider-only) | `@provider-live @shiprocket-live` | Shiprocket live tracking/status provider callback ingestion. | Real tracking callback delivery. |

## CI Policy

- Default CI (`npm run test:e2e:critical`, default journey Playwright shards) is mock/provider-safe.
- Default CI must not call real Razorpay or Shiprocket.
- Provider-live journeys are a separate, explicitly approved run mode.
