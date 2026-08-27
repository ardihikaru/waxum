FROM rust:slim-bookworm AS rust-builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    curl \
    git \
    cmake \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

ENV CMAKE_POLICY_VERSION_MINIMUM=3.5

# rust-toolchain.toml is copied WITH the manifests, and there is deliberately
# no `rustup default nightly` above it. That line used to be here, and it
# silently defeated the pin: rust-toolchain.toml lives in the repo root but
# was never copied into the build context, so inside the image it did not
# exist and cargo fell back to whatever nightly was current on build day.
# On a nightly channel that means an image build could fail on a compiler or
# dependency change unrelated to anything in src/ -- indistinguishable, from
# the error output, from a real bug in this repo. Copying the pin makes a
# green `cargo build --release` on a developer's machine actually predict a
# green image build, because both resolve the same dated toolchain.
COPY Cargo.toml Cargo.lock* rust-toolchain.toml ./

RUN mkdir -p src && \
    echo 'fn main() { println!("dummy"); }' > src/main.rs

RUN cargo build --release 2>/dev/null || true

RUN rm -rf src target/release/waxum target/release/deps/waxum*

COPY src/ ./src/

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    gosu \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/cache/apt/*

COPY --from=rust-builder /app/target/release/waxum /app/waxum
COPY docker-entrypoint.sh /app/docker-entrypoint.sh

RUN mkdir -p /app/whatsapp_sessions \
    && groupadd --system --gid 1000 waxum \
    && useradd --system --uid 1000 --gid waxum --no-create-home --shell /usr/sbin/nologin waxum \
    && chown -R waxum:waxum /app \
    && chmod +x /app/docker-entrypoint.sh

ENV WHATSAPP_STORAGE_PATH=/app/whatsapp_sessions
ENV RUST_LOG=waxum=info,tower_http=info

EXPOSE 3451

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -fsS --max-time 4 http://127.0.0.1:3451/health || exit 1

# No `USER waxum` here: the container starts as root so the entrypoint can
# chown mounted volumes (which may still be owned by root from a
# pre-0.11.1 image) before dropping to the unprivileged `waxum` user via
# gosu to actually run the binary. The process that ends up running the
# app is never root -- only the brief setup step is.
ENTRYPOINT ["/app/docker-entrypoint.sh"]
CMD ["/app/waxum"]
