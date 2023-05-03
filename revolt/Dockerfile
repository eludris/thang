# syntax=docker/dockerfile:1
FROM rust:slim-bullseye as revolt-builder

WORKDIR /revolt

COPY revolt/Cargo.toml Cargo.lock ./
COPY ./revolt/src ./src
COPY ./models /models
COPY ./revolt-wrapper /revolt-wrapper

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/revolt/target \
    cargo build --release
# Other image cannot access the target folder.
RUN --mount=type=cache,target=/revolt/target \
    cp ./target/release/revolt /usr/local/bin/revolt

FROM debian:bullseye-slim

# Remove docker's default of removing cache after use.
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
ENV PACKAGES ca-certificates
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -yqq --no-install-recommends \
    $PACKAGES && rm -rf /var/lib/apt/lists/*

COPY --from=revolt-builder /usr/local/bin/revolt /bin/revolt

CMD ["/bin/revolt"]
