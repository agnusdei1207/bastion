FROM messense/rust-musl-cross:x86_64-musl AS builder
WORKDIR /app
COPY axum/Cargo.toml axum/Cargo.lock ./axum/
COPY axum ./axum
RUN cd axum && cargo build --release

FROM jasonish/suricata:latest
WORKDIR /app
COPY .env /app/.env
COPY --from=builder /app/axum/target/x86_64-unknown-linux-musl/release/axum /app/axum
COPY suricata/suricata.yaml /etc/suricata/suricata.yaml
COPY suricata/rules /var/lib/suricata/rules/suricata.rules
RUN chmod +x /app/axum
EXPOSE 3000

ENTRYPOINT ["/app/axum"]