# syntax=docker/dockerfile:1
FROM rust:slim-bullseye as builder

WORKDIR /eludris

COPY eludris/Cargo.toml Cargo.lock ./
COPY ./eludris/src ./src
COPY ./models /models
COPY ./revolt-wrapper /revolt-wrapper

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES libssl-dev pkg-config
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/eludris/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/eludris/target \
    cp ./target/release/eludris /usr/local/bin/eludris

FROM debian:bullseye-slim

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES libssl-dev pkg-config ca-certificates
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/eludris /bin/eludris

ENV RUST_LOG debug

CMD ["/bin/eludris"]
