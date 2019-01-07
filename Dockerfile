FROM alpine:edge
RUN apk add --no-cache sqlite-dev libseccomp-dev
RUN apk add --no-cache --virtual .build-rust rust cargo
WORKDIR /usr/src/sn0int
COPY . .
RUN cargo build --release --verbose
RUN strip target/release/sn0int

FROM alpine:edge
RUN apk add --no-cache libgcc sqlite-libs libseccomp
COPY --from=0 /usr/src/sn0int/target/release/sn0int /usr/local/bin/sn0int
VOLUME ["/data", "/cache"]
ENV XDG_DATA_HOME=/data \
    XDG_CACHE_HOME=/cache
ENTRYPOINT ["sn0int"]
