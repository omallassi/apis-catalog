#
# Will build the base image (incl. apis-catalog server and script to start it)
#

FROM rust:1.58.1-buster
RUN apt-get update && apt-get -y install gettext-base

VOLUME [ "/data" ]

WORKDIR /apis-catalog
COPY ./ ./
RUN cargo build

RUN chmod a+rwx ./docker-config/entrypoint.sh

#ENTRYPOINT [ "./docker-config/entrypoint.sh" , "--variables"]