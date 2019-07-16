FROM rust:buster
RUN rustup install nightly
WORKDIR /usr/src/sn0int-registry
COPY . .
RUN cargo +nightly build --release --verbose
RUN strip target/release/sn0int-registry

FROM debian:buster
RUN apt-get update -q && apt-get install -yq libcurl4 libpq5 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=0 /usr/src/sn0int-registry/target/release/sn0int-registry /usr/local/bin/sn0int-registry
COPY templates /templates
ENV ROCKET_ENV=prod \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=8000
USER www-data
ENTRYPOINT ["sn0int-registry"]
