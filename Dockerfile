FROM docker.io/library/rust:1.54 AS build

COPY src /app/src
COPY *.toml /app

WORKDIR /app

RUN cargo build --release

FROM docker.io/library/debian:stable-slim

COPY --from=build /app/target/release/ip_exporter /ip_exporter

EXPOSE 3030
ENTRYPOINT [ "/ip_exporter" ]
