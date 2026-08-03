# Logpose - URL Shortener

> Log Pose points your users straight to their destination.

A high-performance, containerized URL shortener API built with **Rust**, **Axum**, **Diesel**, and **PostgreSQL**.

---

## Architecture

This project follows a clean, multi-layered architecture in Rust to keep concerns separated and the codebase easy to reason about and extend.

Requests enter through the route definitions in `src/routes/` and are forwarded to controllers in the handlers layer under `src/handlers/`. Business and database query logic is decoupled from routes and handled in `src/repository/` using the Diesel ORM, while database structures and request/response payloads live in `src/models/`.

Auto-incrementing database IDs are never exposed directly. Instead, short codes are generated using a base62-encoded **Format-Preserving Encryption (FPE)** scheme with **AES-256**, which prevents enumeration of sequential IDs (i.e. nobody can guess `abc123` → `abc124` and walk your entire link table). Other cryptographic helpers - password hashing with **Argon2**, JWT signing/validation - are encapsulated inside `src/utils/` to keep every layer highly cohesive and swappable.

---

## Concepts Applied

- **Layered architecture** - routes → handlers → repository → models, each layer with a single responsibility
- **Format-Preserving Encryption (FPE)** with AES-256 for non-sequential, non-enumerable short codes
- **Base62 encoding** for compact, URL-safe short codes
- **Argon2** password hashing for secure credential storage
- **JWT-based authentication** with custom Axum extractors (`FromRequestParts`)
- **Connection pooling** (r2d2) for efficient database access under load
- **Client IP resolution** via reverse-proxy-aware header parsing (`X-Forwarded-For` / `X-Real-IP`), guarding against spoofing by trusting only the appropriate hop
- **Click analytics capture** - IP, user agent, browser/device parsing, referer, and (planned) geolocation on every redirect
- **Dockerized deployment** - multi-stage build, isolated app + database services via Docker Compose
- **Clean error handling** - no panics on the request path; pool/DB failures degrade gracefully instead of crashing the server

---

## Getting Started (Docker Setup)

1. **Configure Environment Variables**

   Copy `.env` at the root directory and update credentials if needed (it comes pre-configured with sane local defaults):

   ```bash
   # Make sure SERVER_URL is set to 0.0.0.0:8000 so the container can accept external requests
   SERVER_URL=0.0.0.0:8000
   ```

2. **Spin Up the Containers**

   Build the server binary inside the compiler image and bring up the app + database services:

   ```bash
   docker compose up --build -d
   ```

3. **Verify the Installation**

   ```bash
   curl http://localhost:8000/api/health
   ```

---

## API Overview

| Method | Endpoint | Description | Status |
|---|---|---|---|
| `POST` | `/api/auth/signup` | Register a new user | Live |
| `POST` | `/api/auth/login` | Authenticate and receive a JWT | Live |
| `POST` | `/api/urls` | Create a shortened URL | Live |
| `GET` | `/:short_code` | Redirect to the original long URL | Live |
| `GET` | `/api/health` | Service health check | Live |
| `GET` | `/api/urls` | List all URLs for the authenticated user | Planned |
| `GET` | `/api/urls/:id/analytics` | Detailed click analytics for a link | Planned |
| `PATCH` | `/api/urls/:id` | Update destination, expiry, or alias | Planned |
| `DELETE` | `/api/urls/:id` | Deactivate/delete a short link | Planned |
| `POST` | `/api/urls/:id/custom-alias` | Set a custom memorable short code | Planned |
| `POST` | `/api/billing/checkout` | Start a Stripe Checkout session | Planned |
| `POST` | `/api/billing/webhook` | Handle Stripe subscription lifecycle events | Planned |
| `GET` | `/api/billing/portal` | Redirect to Stripe customer billing portal | Planned |
| `POST` | `/api/auth/logout` | Invalidate the active JWT | Planned |

---

## Roadmap

### Core Platform
- [ ] **Custom Shortcodes** - let users define memorable aliases instead of generated codes
- [ ] **URL Expiration** - TTL-based auto-deactivation of links
- [ ] **Rate Limiting** - protect shortcode resolution and auth endpoints from abuse
- [ ] **JWT Blacklisting** - logout support backed by a fast invalidation store
- [ ] **Load Balancing** - distribute traffic across multiple running app instances

### Performance & Scale
- [ ] **Redis Caching Layer** - cache hot short-code → long-URL lookups in Redis to skip the DB entirely on repeat redirect hits, with write-through invalidation on link updates/deletes
- [ ] **Database Sharding** - horizontally partition the `urls` table (e.g. by short-code hash range or user ID) across multiple Postgres instances to scale writes and storage past a single node, with a routing layer to direct queries to the correct shard

### Monetization
- [ ] **Stripe Integration** - subscription billing, Checkout sessions, customer portal, and webhook-driven plan tiers (Free / Pro / Business) gating features like custom domains, analytics retention, and rate limits

### Product & UX
- [ ] **Dashboard UI** - a web dashboard for managing links, viewing click analytics (geolocation, browser, device, referrer breakdowns), and configuring account/billing settings
- [ ] **Frontend Application** - full user-facing web app for link creation, QR code generation, and profile management, consuming the API above

---

## Tech Stack

**Backend:** Rust · Axum · Diesel ORM · PostgreSQL
**Auth & Security:** JWT · Argon2 · AES-256 FPE
**Caching:** Redis *(planned)*
**Payments:** Stripe *(planned)*
**Infra:** Docker · Docker Compose · DB Sharding *(planned)*
**Frontend:** TBD *(planned)*

---

*Named after the Log Pose from One Piece - the compass that doesn't point north, it points to wherever you're meant to go next. This project aims to do the same for your links.*
