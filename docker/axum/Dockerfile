# -------------------------------
# Step 1: 빌드 단계 (정적 빌드)
# -------------------------------
FROM messense/rust-musl-cross:x86_64-musl AS builder

WORKDIR /app

# Cargo.toml, Cargo.lock만 먼저 복사하여 의존성 캐시 활용
COPY axum/Cargo.toml axum/Cargo.lock ./axum/

# 이후 전체 프로젝트 복사
COPY axum ./axum

# 빌드
RUN cd axum && cargo build --release

# -------------------------------
# Step 2: 런타임 단계 (경량 Alpine)
# -------------------------------
FROM alpine:latest

# SSL 통신용 인증서
RUN apk add --no-cache ca-certificates

# 빌드 결과 복사
COPY --from=builder /app/axum/target/x86_64-unknown-linux-musl/release/axum /app/axum

# 실행 권한 부여
RUN chmod +x /app/axum

# 실행
ENTRYPOINT ["/app/axum"]
