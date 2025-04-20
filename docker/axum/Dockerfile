FROM messense/rust-musl-cross:x86_64-musl-amd64 AS builder
WORKDIR /app
COPY axum ./axum
RUN cd axum && cargo build --release
# axum 내부에서 docker를 사용하기 위해 docker.io를 설치합니다.
RUN apt-get update && apt-get install -y docker.io

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/axum/target/x86_64-musl-amd64-unknown-linux-musl/release/axum /app/axum
RUN chmod +x /app/axum
EXPOSE 3000

ENTRYPOINT ["/app/axum"]