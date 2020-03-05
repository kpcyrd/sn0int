FROM rust:buster
RUN apt-get update -q && apt-get install -yq libsqlite3-dev libseccomp-dev libsodium-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/sn0int
COPY . .
RUN cargo build --release --verbose
RUN strip target/release/sn0int

FROM debian:buster
RUN apt-get update -q && apt-get install -yq libsqlite3-dev libseccomp-dev libsodium-dev \
    && rm -rf /var/lib/apt/lists/*
COPY --from=0 /usr/src/sn0int/target/release/sn0int /usr/local/bin/sn0int
VOLUME ["/data", "/cache"]
ENV XDG_DATA_HOME=/data \
    XDG_CACHE_HOME=/cache
ENTRYPOINT ["sn0int"]
