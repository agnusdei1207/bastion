# ---------------------------------------
# Step 1: Rust 환경에서 Axum 빌드
# ---------------------------------------
FROM messense/rust-musl-cross:x86_64-musl AS builder

WORKDIR /app

# 의존성 캐싱을 위해 먼저 Cargo.toml과 Cargo.lock 복사
COPY axum/Cargo.toml axum/Cargo.lock ./axum/
WORKDIR /app/axum
RUN cargo build --release && \
    rm ./target/release/deps/axum*

# 실제 소스 코드 복사 후 빌드
COPY axum/src ./src
RUN cargo build --release && \
    strip ./target/x86_64-unknown-linux-musl/release/axum

# ---------------------------------------
# Step 2: 최종 이미지 구성
# ---------------------------------------
FROM alpine:3.18

# Suricata 설치
RUN apk add --no-cache suricata jq && \
    mkdir -p /var/log/suricata && \
    chown -R suricata:suricata /var/log/suricata

# 앱 디렉토리 및 로그 디렉토리 생성
WORKDIR /app
RUN mkdir -p /app/suricata_logs

# 필요한 파일 복사
COPY --from=builder /app/axum/target/x86_64-unknown-linux-musl/release/axum /app/axum
COPY .env /app/.env
COPY suricata.yaml /etc/suricata/suricata.yaml

# 실행 권한 부여
RUN chmod +x /app/axum

# 포트 노출
EXPOSE 3000

# 헬스체크 스크립트 추가
COPY healthcheck.sh /app/
RUN chmod +x /app/healthcheck.sh

# Suricata 사용자로 전환하지 않고 root로 실행 (필요시 변경)
USER root

# 멀티프로세스 관리를 위한 스크립트 사용
COPY entrypoint.sh /app/
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]