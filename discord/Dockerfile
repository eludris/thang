FROM rust:1.65-slim-buster as build

RUN USER=root cargo new --bin discord
WORKDIR /discord

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY discord/Cargo.toml Cargo.lock ./
COPY ./models /models

RUN cargo build --release
RUN rm src/*.rs

COPY ./discord/src ./src

RUN rm ./target/release/deps/discord*
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /discord/target/release/discord /usr/src/discord

CMD ["/usr/src/discord"]
