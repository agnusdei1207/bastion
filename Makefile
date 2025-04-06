# --------- M1 Mac 환경 전용 ---------

# musl 크로스 컴파일러 설치 (x86_64 리눅스용 빌드 위해 필요)
install:
	brew install FiloSottile/musl-cross/musl-cross


# --------- Axum 서비스 관련 ---------

# 개발 모드로 실행 (로컬 Mac에서 실행)
run-axum:
	cd axum && cargo run

# 빌드 결과물 정리 (target 디렉토리 삭제)
clean-axum:
	cd axum && cargo clean

# 릴리즈 빌드 후 실행 (Mac에서 실행됨)
release-axum:
	cd axum && cargo build --release && \
	ls -lh target/release/axum && \
	./target/release/axum

# 리눅스 x86_64 (AMD64) 용 MUSL 정적 링크 릴리즈 빌드
# -> 배포 가능한 리눅스 바이너리 생성
release-axum-musl:
	cd axum && \
	OPENSSL_STATIC=1 \
	OPENSSL_DIR=$(brew --prefix openssl@3) \
	CC=x86_64-linux-musl-gcc \
	CXX=x86_64-linux-musl-g++ \
	cargo build --release --target x86_64-unknown-linux-musl

# ARM64 (예: Raspberry Pi 등) 용 바이너리 실행 (이미 빌드된 경우)
run-release-arm:
	./axum/target/release/axum


# --------- Docker 이미지 관련 ---------

# axum 서비스 Docker 이미지 빌드 후 레지스트리에 푸시
push-axum:
	./docker/axum/push.sh

# fluentd 이미지 푸시
push-fluentd:
	./docker/fluentd/push.sh

# suricata 이미지 푸시
push-suricata:
	./docker/suricata/push.sh


# --------- Docker Compose 환경 ---------

# 전체 서비스 컨테이너 실행
docker-compose-up:
	docker-compose -f docker/docker-compose.yml up -d

# 전체 서비스 컨테이너 중지 및 볼륨 제거
docker-compose-down:
	docker-compose -f docker/docker-compose.yml down -v
