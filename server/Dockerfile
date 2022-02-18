FROM rust:1.58.1-buster

RUN apt-get update && apt-get -y install gettext-base

COPY ./ ./

RUN cargo build

ENTRYPOINT  envsubst < "./config/local-template.toml" > "./config/local-bis.toml" && RUST_LOG="info,apis_catalog_server=debug" ./target/debug/apis_catalog_server &