# ezproxy

A simple HTTP proxy written in **Rust** using the `hyper` and `tower` crates.

## Overview

The proxy validates an incoming request's `Authorization` header against a secret token. If the token matches, the request is forwarded to an upstream server; otherwise a **401 Unauthorized** response is returned.

Key features:

- Auth middleware using an environment variable (`AUTH_TOKEN`).
- Configurable upstream target via `UPSTREAM_URL`.
- Configurable bind address (`BIND_ADDR`, defaults to `127.0.0.1:3000`).
- Built on top of **hyper** (HTTP client/server) and **tower** for future extensibility.

## Getting Started

### Prerequisites

- Rust toolchain (`cargo` & `rustc`).
- GitHub CLI (`gh`) – already used to set up the repository.

### Build & Run

```bash
# Clone (if you haven't already)
git clone https://github.com/matsilva/ezproxy.git
cd ezproxy

# Set required environment variables
export AUTH_TOKEN="my-secret-token"
export UPSTREAM_URL="http://example.com"   # Target upstream server
# Optional: custom listen address
# export BIND_ADDR="0.0.0.0:8080"

# Build and run
cargo run
```

The proxy will start listening on `127.0.0.1:3000` (or the address you set).

### Making a Request

```bash
curl -H "Authorization: my-secret-token" http://localhost:3000/some/path?query=val
```

- If the token matches, the request is proxied to `UPSTREAM_URL` preserving the path and query.
- If missing or incorrect, you receive a **401 Unauthorized** response.

## Extending the Proxy

- Add TLS support with `hyper-tls` for HTTPS upstreams.
- Replace simple token check with JWT validation or OAuth.
- Integrate logging/tracing (`tracing`, `env_logger`).
- Write integration tests using `reqwest` or similar client libraries.

## License

MIT © 2025 matsilva
