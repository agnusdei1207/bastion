# 📦 개발 환경에서 Axum 실행 (디버그 모드)
run:
	cd axum && cargo run

# 🧹 빌드 아티팩트 정리
clean:
	cd axum && cargo clean

# 🚀 릴리즈 모드로 빌드 (현재 머신 아키텍처 기준)
release:
	cd axum && cargo build --release && ls -lh axum/target/release/axum

# 🚀 릴리즈 빌드 실행
run-release:
	./axum/target/release/axum

# 🛠️ x86_64 아키텍처용 릴리즈 빌드 (M1에서 실행 후 x86_64 배포를 위한 경우)
x86_64-release:
	cd axum && \
	rustup target add x86_64-unknown-linux-musl && \
	cargo build --release --target x86_64-unknown-linux-musl && \
	ls -lh axum/target/x86_64-unknown-linux-musl/release/axum

# 🐳 도커 이미지 빌드 (로컬에서 빌드된 바이너리 포함)
docker-build-axum:
	docker build -t agnusdei1207/axum:latest .
