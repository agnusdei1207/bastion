# Step 1: Build stage using musl for static linking (Rust -> MUSL)
FROM messense/rust-musl-cross:aarch64-musl as builder

# 작업 디렉터리 설정
WORKDIR /app

# 의존성 캐시 활용을 위한 선별 복사
COPY ./axum/Cargo.toml ./axum/Cargo.lock ./axum/

# dummy main.rs로 먼저 의존성만 빌드 (선택 사항, 속도 향상용)
RUN mkdir -p ./axum/src && echo "fn main() {}" > ./axum/src/main.rs
RUN cd axum && cargo build --release
RUN rm -f ./axum/src/main.rs

# 전체 소스 코드 복사 후 재빌드
COPY ./axum ./axum

# 최종 릴리즈 빌드
RUN cd axum && cargo build --release

# Step 2: Runtime stage (Alpine Linux - 경량)
FROM alpine:latest

# 루트 인증서 설치 (TLS 통신 등 필요시)
RUN apk add --no-cache ca-certificates

# 빌드된 실행 파일만 복사 (정적 링크이므로 이것만 있으면 실행 가능)
COPY --from=builder /app/axum/target/aarch64-unknown-linux-musl/release/axum /app/axum

# 실행 권한 부여
RUN chmod +x /app/axum

# 컨테이너가 실행될 때 바로 axum 실행
ENTRYPOINT ["/app/axum"]
