# syntax=docker/dockerfile:1.7-labs
FROM rust:1.90-alpine AS builder

ARG TARGETPLATFORM
ARG BINARY_NAME=pico_limbo

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
    cargo build --release --target $TARGET --bin $BINARY_NAME && \
    cp target/$TARGET/release/$BINARY_NAME /usr/local/bin/pico_limbo

FROM gcr.io/distroless/static:latest

COPY --from=builder --chown=nonroot:nonroot /usr/src/app /usr/src/app

WORKDIR /usr/src/app

COPY --from=builder --chown=nonroot:nonroot /usr/local/bin/pico_limbo /usr/local/bin/pico_limbo

USER nonroot

CMD ["pico_limbo"]
