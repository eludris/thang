FROM rust:1.63-slim-buster as build

RUN USER=root cargo new --bin thang-discord
WORKDIR /thang-discord

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/thang_discord*
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /thang-discord/target/release/thang-discord /usr/src/thang-discord

CMD ["/usr/src/thang-discord"]