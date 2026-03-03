FROM rust:1.89.0-slim-bookworm as builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

# build app and diesel CLI
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo build --release 

# runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libpq5 && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/dns-service

# copy app and diesel
COPY --from=builder /usr/src/app/target/release/dns_scriptylabs_com .
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/
COPY --from=builder /usr/src/app/migrations /opt/dns-service/migrations
COPY --from=builder /usr/src/app/.env /opt/dns-service/.env

COPY start.sh /opt/dns-service/start.sh
RUN chmod +x /opt/dns-service/start.sh ./dns_scriptylabs_com
RUN chmod +x /opt/dns-service/migrations/
RUN chmod +x /opt/dns-service/dns_scriptylabs_com


EXPOSE 5550

CMD ["/opt/dns-service/start.sh"]
