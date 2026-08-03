# Rust URL Shortener

A high-performance, containerized URL shortener API built with Axum, Diesel, and PostgreSQL.

## Architecture

This project is built using a clean, multi-layered architecture in Rust to ensure separate concerns and easy maintainability. Requests enter through the route definitions in `src/routes/` and are forwarded to controllers in the handlers layer under `src/handlers/`. Business and database query logic is decoupled from routes and handled in `src/repository/` using the Diesel ORM, while database structures and request/response payloads are managed in the `src/models/` layer. Obfuscation of auto-incrementing database IDs is done using a base62-encoded Format-Preserving Encryption (FPE) scheme with AES-256, which prevents enumeration of short codes, and other cryptographic helper functions (such as password hashing with Argon2 and JWT validation) are encapsulated inside `src/utils/` to keep layers highly cohesive.

## Getting Started (Docker Setup)

Follow these steps to set up and run the service locally using Docker Compose:

1. **Configure Environment Variables**:
   Copy `.env` at the root directory and update credentials if needed (it comes pre-configured with production-ready default keys):
   ```bash
   # Make sure SERVER_URL is set to 0.0.0.0:8000 so the container can accept external requests
   SERVER_URL=0.0.0.0:8000
   ```

2. **Spin Up the Containers**:
   Build the server binary inside the compiler image container and run the app and database services:
   ```bash
   docker compose up --build -d
   ```

3. **Verify the Installation**:
   Ensure the service is up and running by querying the health endpoint:
   ```bash
   curl http://localhost:8000/api/health
   ```

## Upcoming & TODOs

- [ ] **JWT Blacklisting**: Implement a logout handler and a cache store (e.g., Redis) to invalidate active tokens before expiration.
- [ ] **Rate Limiting**: Integrate rate-limiting middleware to secure shortcode resolution and signup endpoints.
- [ ] **Custom Shortcodes**: Allow users to define custom, memorable alias keys instead of automatically generated hash codes.
- [ ] **Analytics Dashboard**: Develop a frontend dashboard displaying advanced metrics like geolocations, browsers, and referrers.
- [ ] **URL Expiration**: Add expiration dates to URLs so links automatically deactivate after a configured time-to-live (TTL).
