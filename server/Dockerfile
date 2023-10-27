FROM rust:1.73.0-bookworm AS builder
WORKDIR /apis-catalog
COPY ./ ./
#RUN cargo install --path .
RUN cargo build


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y gettext-base openssl git && rm -rf /var/lib/apt/lists/*
#COPY --from=builder /usr/local/cargo/bin/apis_catalog_server /usr/local/bin/apis_catalog_server

WORKDIR /apis-catalog
COPY --from=builder /apis-catalog/target/debug/apis_catalog_server ./
COPY --from=builder /apis-catalog/config/*.toml ./config/
COPY --from=builder /apis-catalog/docker-config/entrypoint.sh ./docker-config/
RUN chmod a+rwx ./docker-config/entrypoint.sh

VOLUME [ "/data" ]