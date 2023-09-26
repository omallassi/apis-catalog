#
# Will build the base image (incl. apis-catalog server and script to start it)
#

FROM rust:1.72.1-buster AS builder
RUN apt-get update

WORKDIR /apis-catalog
COPY ./ ./
RUN cargo build

#
#
#
FROM rust:1.72.1-buster AS runner
RUN apt-get update && apt-get -y install gettext-base

WORKDIR /apis-catalog
COPY --from=builder /apis-catalog/target/debug/apis_catalog_server ./
COPY --from=builder /apis-catalog/config/*.toml ./config/
COPY --from=builder /apis-catalog/docker-config/entrypoint.sh ./docker-config/

RUN chmod a+rwx ./docker-config/entrypoint.sh

VOLUME [ "/data" ]

#ENTRYPOINT [ "./docker-config/entrypoint.sh" , "--variables"]