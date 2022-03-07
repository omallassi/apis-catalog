FROM rust:1.58.1-buster

RUN apt-get update && apt-get -y install gettext-base

COPY ./ ./

RUN cargo build

RUN chmod a+rwx ./docker-config/entrypoint.sh

ENTRYPOINT [ "./docker-config/entrypoint.sh" , "--variables"]