FROM rust:alpine3.11
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev sqlite-dev libseccomp-dev libsodium-dev
WORKDIR /usr/src/sn0int
COPY . .
RUN cargo build --release --verbose
RUN strip target/release/sn0int

FROM alpine:3.11
RUN apk add --no-cache libgcc sqlite-libs libseccomp libsodium
COPY --from=0 /usr/src/sn0int/target/release/sn0int /usr/local/bin/sn0int
VOLUME ["/data", "/cache"]
ENV XDG_DATA_HOME=/data \
    XDG_CACHE_HOME=/cache
ENTRYPOINT ["sn0int"]
