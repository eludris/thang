FROM rust:1.61 as build

RUN USER=root cargo new --bin thang
WORKDIR /thang

COPY Cargo.lock Cargo.toml ./

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/thang*
RUN cargo build --release

FROM debian:buster-slim

COPY --from=build /thang/target/release/thang /usr/src/thang

CMD ["/usr/src/thang"]
