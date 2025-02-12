FROM rust:1.84-alpine AS build

RUN apk add --no-cache musl-dev

WORKDIR /usr/src/app
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/app/target \
    cargo build --release --locked && \
    cp /usr/src/app/target/release/pico_limbo /usr/bin/pico_limbo

FROM alpine

WORKDIR /usr/src/app

COPY data/generated /usr/src/app/data
COPY --from=build /usr/bin/pico_limbo /usr/bin/pico_limbo

ENV DATA_DIR=/usr/src/app/data

CMD ["pico_limbo", "-a", "0.0.0.0:25565"]
