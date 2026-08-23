# syntax=docker/dockerfile:1

ARG APP_NAME=logpose

# Build stage
FROM amazonlinux:2023 AS build
ARG APP_NAME
WORKDIR /app

RUN dnf update -y && \
    dnf install -y --allowerasing \
    gcc \
    gcc-c++ \
    make \
    clang \
    git \
    openssl-devel \
    libpq-devel \
    pkgconfig \
    perl \
    tar \
    gzip \
    curl && \
    dnf clean all

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal

# Pre-cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --locked --release && \
    rm -rf src

# Copy source code and build production release binary
COPY src ./src
RUN touch src/main.rs && \
    cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server && \
    strip /bin/server

# Final lightweight production runtime stage
FROM amazonlinux:2023 AS final

RUN dnf update -y && \
    dnf install -y --allowerasing \
    openssl \
    libpq \
    ca-certificates \
    curl \
    shadow-utils && \
    dnf clean all

ARG UID=10001
RUN useradd -u ${UID} -m -s /sbin/nologin appuser
USER appuser

COPY --from=build /bin/server /bin/server

ENV SERVER_URL=0.0.0.0:8000 \
    RUST_LOG=info

EXPOSE 8000

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/api/health || exit 1

CMD ["/bin/server"]
