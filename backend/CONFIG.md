# Configuration & Deployment

## Environment variables

See [.env.example](.env.example) for the full list of required and optional variables (database, gRPC, OAuth, Redis, R2, GraphQL bind address, logging).

**Production:** Do not use example or default secrets in production. Override all sensitive values via environment variables or a secret manager (e.g. Kubernetes secrets, AWS Secrets Manager). The values in `.env.example` are placeholders only.

## Health endpoints (GraphQL service)

For orchestrators (e.g. Kubernetes):

- **Liveness — `GET /`**  
  Returns 200 if the process is running. Use for restart decisions (e.g. `livenessProbe`). No dependency checks.

- **Readiness — `GET /ready`**  
  Returns 200 if the service can serve traffic: checks gRPC backend (and thus DB, via the gRPC Readiness RPC) and, if `REDIS_URL` is set, Redis. Returns 503 if any configured check fails. Use for traffic routing (e.g. `readinessProbe`) so the pod is not sent requests until dependencies are up.

## Bind addresses

- **GraphQL:** `GRAPHQL_LISTEN_ADDR` (default `0.0.0.0:8080`).
- **gRPC (core_operations):** `GRPC_SERVER` (default `0.0.0.0:50051`).
