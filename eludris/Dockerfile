FROM rust:1.65-slim-buster as build

RUN USER=root cargo new --bin eludris
WORKDIR /eludris

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY eludris/Cargo.toml Cargo.lock ./
COPY ./models /models

RUN cargo build --release
RUN rm src/*.rs

COPY ./eludris/src ./src

RUN rm ./target/release/deps/eludris*
RUN cargo build --release

FROM debian:buster-slim

RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

COPY --from=build /eludris/target/release/eludris /usr/src/eludris

CMD ["/usr/src/eludris"]
