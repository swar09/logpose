# syntax=docker/dockerfile:1

ARG APP_NAME=logpose

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

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release && \
    cp ./target/release/$APP_NAME /bin/server

FROM amazonlinux:2023 AS final

# Install runtime libraries
RUN dnf update -y && \
    dnf install -y --allowerasing \
    openssl \
    libpq \
    ca-certificates \
    shadow-utils && \
    dnf clean all

ARG UID=10001
RUN useradd -u ${UID} -m -s /sbin/nologin appuser
USER appuser

COPY --from=build /bin/server /bin/server

EXPOSE 8000

CMD ["/bin/server"]
