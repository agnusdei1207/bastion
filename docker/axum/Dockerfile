FROM messense/rust-musl-cross:x86_64-musl
RUN rustup update beta && rustup target add --toolchain beta x86_64-unknown-linux-musl
WORKDIR /app
COPY axum/Cargo.toml axum/Cargo.lock ./axum/
COPY axum ./axum
RUN cd axum && cargo build --release

FROM jasonish/suricata:latest
WORKDIR /app
COPY .env /app/.env
COPY --from=builder /app/axum/target/x86_64-unknown-linux-musl/release/axum /app/axum
RUN chmod +x /app/axum

RUN apk add --no-cache tree
RUN tree -L 3 /
RUN tree -L 3 /etc

EXPOSE 3000

ENTRYPOINT ["/app/axum"]