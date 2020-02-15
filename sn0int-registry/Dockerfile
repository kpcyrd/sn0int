FROM rust:buster
RUN apt-get update -q && apt-get install -yq llvm libclang-dev \
    && rm -rf /var/lib/apt/lists/*
RUN rustup install nightly
WORKDIR /usr/src/sn0int
COPY . .
RUN cd sn0int-registry && cargo +nightly build --release --verbose
RUN strip target/release/sn0int-registry

FROM debian:buster
RUN apt-get update -q && apt-get install -yq libcurl4 libpq5 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=0 /usr/src/sn0int/target/release/sn0int-registry /usr/local/bin/sn0int-registry
COPY sn0int-registry/templates /templates
ENV ROCKET_ENV=prod \
    ROCKET_ADDRESS=0.0.0.0 \
    ROCKET_PORT=8000
USER www-data
ENTRYPOINT ["sn0int-registry"]
