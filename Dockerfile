# syntax=docker/dockerfile:1.7-labs
FROM rust:1.90-alpine AS builder

ARG TARGETPLATFORM
ARG BINARY_NAME=pico_limbo
ARG RUST_FEATURES=""

WORKDIR /usr/src/app
COPY --parents ./Cargo.toml ./Cargo.lock ./crates ./pico_limbo ./data/generated ./

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    apk add --no-cache musl-dev && \
    case "${TARGETPLATFORM}" in \
        linux/amd64) TARGET="x86_64-unknown-linux-musl";; \
        linux/arm64) TARGET="aarch64-unknown-linux-musl";; \
        *) echo "Unsupported platform: ${TARGETPLATFORM}"; exit 1;; \
    esac && \
    rustup target add $TARGET && \
    FEATURES_FLAG="" && \
    if [ -n "$RUST_FEATURES" ]; then \
        FEATURES_FLAG="--features ${RUST_FEATURES}"; \
    fi && \
    cargo build --release --target $TARGET --bin $BINARY_NAME $FEATURES_FLAG && \
    cp target/$TARGET/release/$BINARY_NAME /usr/local/bin/pico_limbo


FROM gcr.io/distroless/static:latest

WORKDIR /usr/src/app

COPY --from=builder /usr/local/bin/pico_limbo /usr/local/bin/pico_limbo

CMD ["/usr/local/bin/pico_limbo"]
