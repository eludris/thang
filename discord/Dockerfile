# syntax=docker/dockerfile:1
FROM rust:slim-bullseye as discord-builder

WORKDIR /discord

COPY discord/Cargo.toml Cargo.lock ./
COPY ./discord/src ./src
COPY ./models /models

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/discord/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/discord/target \
    cp ./target/release/discord /usr/local/bin/discord

FROM debian:bullseye-slim

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES ca-certificates
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

COPY --from=discord-builder /usr/local/bin/discord /bin/discord

CMD ["/bin/discord"]
