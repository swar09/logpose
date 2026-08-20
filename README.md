# Logpose - URL Shortener

> *Log Pose points your users straight to their destination.*

Logpose is a high-performance, scalable URL shortener built with Rust, the Axum framework, Diesel ORM, PostgreSQL and Redis. This project follows a clean, multi-layered architecture in Rust influenced by Domain-Driven Design (DDD) patterns. Requests enter through the route definitions in `src/routes/` and are forwarded to controllers in the handlers layer under `src/handlers/`. Business and database query logic is decoupled from routes and handled in `src/repository/` using the Diesel ORM, while database structures and request/response payloads live in `src/models/`.

I have implemented Redis caching using the `redis` crate. I am using `redis::aio::ConnectionManager`, which automatically reconnects upon Redis restarts or failures, and gracefully falls back to the database whenever Redis is inactive. The application state is wrapped inside an `Arc<AppState>`, which shares the Redis ConnectionManager, PostgreSQL ConnectionManager (`r2d2`), and other shared state variables across the backend.

For short code generation, I chose a base conversion approach. When a URL is inserted into PostgreSQL, the row index is retrieved via PostgreSQL's `SERIAL` type. The database index is then obfuscated and shifted using AES-256 Format-Preserving Encryption (FF1), and converted into a 4-letter base62 short code before being saved to the database. During lookup, this process is reversed to achieve $O(1)$ time complexity. This approach is completely collision-free, so there is no need for collision resolution overhead.

When a user opens `/{short_code}`, they are temporarily redirected. The redirect handler spawns an asynchronous background task (via Tokio) that extracts and logs click analytics—including IP address, user agent, browser, device, and referrer—without delaying the redirect response.

The database is indexed by the serial ID, the short code for rapid lookups, and by email in the users table. Using Diesel ORM allows writing type-safe queries in Diesel's DSL rather than raw SQL, while Diesel takes care of query generation, execution, and error handling.

For networking and security, Nginx is configured as a reverse proxy load balancer using round-robin distribution across multiple app instances. Nginx also forwards client headers (`X-Real-IP`, `X-Forwarded-For`), which are required by the downstream Token Bucket rate limiter so it inspects real client IP addresses instead of the proxy's IP. Authentication and security are handled using JWT and Argon2id for password hashing, along with Google OAuth 2.0 login. In addition to automatic short code generation, the service supports 6–30 character custom aliases, instant QR code generation, guest shortening with cookie persistence and automatic registration claiming, as well as a tiered subscription model (Free, Pro, Enterprise) integrated with Razorpay for checkout orders, subscription management, and webhook reconciliation.

Moving forward, I am planning features like database sharding and magic link login/signup methods.

```mermaid
flowchart TB
    %% Client Tier
    subgraph Clients["Clients and Consumers"]
        Browser["Web Browser / Client"]
        APIClient["API Client / Mobile App"]
    end

    %% Edge / Load Balancing Tier
    subgraph EdgeLayer["Reverse Proxy and Load Balancing"]
        NGINX["Nginx Load Balancer (:80)<br/>Upstream: rust_backend<br/>Header Forwarding"]
    end

    %% Application Cluster Tier
    subgraph AppCluster["Axum Backend Application Cluster"]
        Server1["server-1 (:8001 -> 8000)"]
        Server2["server-2 (:8002 -> 8000)"]
        Server3["server-3 (:8003 -> 8000)"]
    end

    %% Application Internals
    subgraph MiddlewareLayer["Middleware and Pipeline"]
        CORS["CorsLayer"]
        RL["RateLimiterLayer (Token Bucket)"]
        AuthExtract["Auth Extractor (JWT FromRequestParts)"]
    end

    subgraph RouteLayer["Routing Layer (src/routes)"]
        R_Auth["/api/v1/auth (Signup, Login, OAuth)"]
        R_User["/api/v1/users (Profile)"]
        R_Url["/api/v1/urls (Shorten, Custom, QR, Manage)"]
        R_Bill["/api/v1/billing (Checkout, Webhooks)"]
        R_Redir["/:short_code (Fast Redirect)"]
        R_Health["/api/health (Health Check)"]
    end

    subgraph HandlerLayer["Handlers (src/handlers)"]
        H_Auth["AuthHandler"]
        H_User["UserHandler"]
        H_Url["UrlHandler"]
        H_Bill["BillingHandler"]
        H_Analytics["UrlAnalyticsHandler"]
    end

    subgraph ServiceLayer["Service Layer (src/service)"]
        S_Url["UrlService (Cache-First Lookup / Create)"]
        S_Rate["Rate Limiter Service"]
    end

    subgraph CryptoLayer["Security and Utilities (src/utils)"]
        FPE["AES-256 FF1 Format-Preserving Encryption"]
        Base62["Base62 Encoder / Decoder"]
        Argon2["Argon2 Password Hasher"]
        JWTUtil["JWT Token Manager"]
        QRGen["QR Code Generator"]
        UAEngine["Analytics Parser (IP, Device, Browser)"]
    end

    subgraph RepoLayer["Repository Layer (src/repository)"]
        Repo_User["UserRepository"]
        Repo_Url["UrlRepository"]
        Repo_Bill["Billing & Webhook Repository"]
        Repo_Analytics["UrlAnalyticsRepository"]
        Pool["Diesel r2d2 Pool"]
    end

    %% External Services
    subgraph ExternalServices["External Services"]
        GoogleAuth["Google OAuth 2.0 API"]
        Razorpay["Razorpay Billing & Webhooks API"]
    end

    %% Data & Storage Tier
    subgraph CacheTier["In-Memory Cache"]
        Redis[("Redis 7 Alpine (:6379)<br/>short_code -> long_url")]
    end

    subgraph DatabaseTier["PostgreSQL 17 Database (:5432)"]
        T_Users[("users table")]
        T_Urls[("urls table")]
        T_Analytics[("url_analytics table")]
        T_Plans[("plans & subscriptions tables")]
        T_Trans[("payments & webhook_events tables")]
    end

    %% Connections - Client to Edge
    Browser -->|HTTP Requests| NGINX
    APIClient -->|API Calls / Redirects| NGINX

    %% Edge to Server Instances
    NGINX --> Server1
    NGINX --> Server2
    NGINX --> Server3

    %% Server Instances to Pipeline
    Server1 --> CORS
    Server2 --> CORS
    Server3 --> CORS

    %% Pipeline to Routing
    CORS --> RL
    RL --> AuthExtract
    AuthExtract --> RouteLayer

    %% Route to Handlers
    R_Auth --> H_Auth
    R_User --> H_User
    R_Url --> H_Url
    R_Bill --> H_Bill
    R_Redir --> S_Url
    R_Health --> HandlerLayer

    %% Handlers to Services and Utils
    H_Auth --> Argon2
    H_Auth --> JWTUtil
    H_Auth --> Repo_User
    H_Auth -.-> GoogleAuth
    H_User --> Repo_User
    H_Url --> S_Url
    H_Url --> UAEngine
    H_Url --> QRGen
    H_Bill --> Repo_Bill
    H_Bill -.-> Razorpay
    H_Analytics --> Repo_Analytics

    %% Service to Crypto and Repositories
    S_Url --> FPE
    FPE --> Base62
    S_Url --> Repo_Url

    %% Service / Handlers to Data Stores
    S_Url -->|1. Cache Hit / Miss| Redis
    Repo_User --> Pool
    Repo_Url --> Pool
    Repo_Bill --> Pool
    Repo_Analytics --> Pool

    %% Pool to DB Tables
    Pool --> T_Users
    Pool --> T_Urls
    Pool --> T_Analytics
    Pool --> T_Plans
    Pool --> T_Trans
```

```bash
# Debug build 
cargo build 

# Release build 
cargo build --release 

# Test
cargo test 

# Linting and formatting 
cargo fmt && cargo clippy --fix 
```

Run in Docker:

```bash
docker compose up -d 
```

## References & Crates Used

* [axum](https://crates.io/crates/axum) - Ergonomic and modular web application framework for Rust.
* [tokio](https://crates.io/crates/tokio) - Asynchronous runtime for the Rust programming language.
* [razorpay-rs](https://crates.io/crates/razorpay-rs) - Client SDK for integrating Razorpay payment and subscription APIs.
* [diesel](https://crates.io/crates/diesel) - Safe, extensible ORM and Query Builder for Rust.
* [redis](https://crates.io/crates/redis) - Redis client library for Rust with async and connection manager support.
* [fpe](https://crates.io/crates/fpe) - Format-Preserving Encryption (FF1 mode) implementation.
* [aes](https://crates.io/crates/aes) - Pure Rust implementation of the Advanced Encryption Standard.
* [base62](https://crates.io/crates/base62) - Fast Base62 encoder and decoder for URL-safe identifiers.
* [jsonwebtoken](https://crates.io/crates/jsonwebtoken) - JSON Web Token (JWT) implementation in Rust.
* [argon2](https://crates.io/crates/argon2) - Password hashing algorithm adhering to the PHC string format.
* [woothee](https://crates.io/crates/woothee) - User-agent string parser for browser, OS, and device classification.
* [tower-http](https://crates.io/crates/tower-http) / [tower](https://crates.io/crates/tower) - Modular HTTP middleware (CORS, rate limiting, service abstractions).
* [tracing](https://crates.io/crates/tracing) & [tracing-subscriber](https://crates.io/crates/tracing-subscriber) - Structured logging and diagnostics framework.
* [reqwest](https://crates.io/crates/reqwest) - Ergonomic HTTP client for third-party service calls and payment APIs.
* [oauth2](https://crates.io/crates/oauth2) - Strongly-typed Rust OAuth2 client library for Google authentication.
* [qrcode](https://crates.io/crates/qrcode) - QR code encoder for generating quick-access QR codes.
