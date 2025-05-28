# syntax=docker/dockerfile:1.7-labs
FROM rust:1.86-alpine AS builder

ARG TARGETPLATFORM
ARG BINARY_NAME=pico_limbo

WORKDIR /usr/src/app
COPY --parents ./Cargo.toml ./Cargo.lock ./crates ./binaries ./data/generated ./

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
    cp target/$TARGET/release/$BINARY_NAME /usr/local/bin/app

FROM alpine

RUN addgroup -S picolimbo && adduser -S picolimbo -G picolimbo
USER picolimbo

WORKDIR /usr/src/app

COPY data/generated /usr/src/app/data
COPY --from=builder /usr/local/bin/app /usr/local/bin/app

ENV DATA_DIR=/usr/src/app/data

CMD ["app", "-a", "0.0.0.0:25565"]
