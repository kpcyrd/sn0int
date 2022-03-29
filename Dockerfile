FROM rust:alpine3.15
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN apk add --no-cache musl-dev sqlite-dev libseccomp-dev libsodium-dev
WORKDIR /usr/src/sn0int
COPY . .
RUN --mount=type=cache,target=/var/cache/buildkit \
    CARGO_HOME=/var/cache/buildkit/cargo \
    CARGO_TARGET_DIR=/var/cache/buildkit/target \
    cargo build --release --locked --verbose && \
    cp -v /var/cache/buildkit/target/release/sn0int /
RUN strip /sn0int

FROM alpine:3.15
RUN apk add --no-cache libgcc sqlite-libs libseccomp libsodium
COPY --from=0 /sn0int /usr/local/bin/sn0int
VOLUME ["/data", "/cache"]
ENV XDG_DATA_HOME=/data \
    XDG_CACHE_HOME=/cache
ENTRYPOINT ["sn0int"]
