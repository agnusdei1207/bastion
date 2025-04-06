# ---------------------------------------
# Step 1: Rust 환경에서 Axum 빌드
# ---------------------------------------
FROM messense/rust-musl-cross:x86_64-musl AS builder

WORKDIR /app

COPY axum/Cargo.toml axum/Cargo.lock ./axum/
COPY axum ./axum
RUN cd axum && cargo build --release

# ---------------------------------------
# Step 2: Suricata 설치된 이미지 사용
# (예: jasonish/suricata:7.0.3)
# ---------------------------------------
FROM jasonish/suricata:latest
WORKDIR /app

# .env 등 설정 파일 복사
COPY .env /app/.env

# 빌드된 바이너리 복사
COPY --from=builder /app/axum/target/x86_64-unknown-linux-musl/release/axum /app/axum

RUN chmod +x /app/axum

# 포트 노출
EXPOSE 3000

ENTRYPOINT ["/app/axum"]
