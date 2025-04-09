# axum
run-axum:
	cd axum && cargo run

clean-axum:
	cd axum && cargo clean

release-axum:
	cd axum && cargo build --release && ls -lh target/release/axum && ./target/release/axum

run-release:
	./axum/target/release/axum

# docker
push-axum:
	./docker/axum/push.sh

push-fluentd:
	./docker/fluentd/push.sh

push-suricata:
	./docker/suricata/push.sh

push-all:
	./docker/axum/push.sh
	./docker/fluentd/push.sh
	./docker/suricata/push.sh

# docker-compose
docker-compose-up:
	docker-compose -f docker/docker-compose.yml up -d
docker-compose-down:
	docker-compose -f docker/docker-compose.yml down -v

# ssh
ssh:
	ssh -i t2.pem ubuntu@216.47.98.209
scp:
	scp -i t2.pem docker/docker-compose.yml ubuntu@216.47.98.209:/home/ubuntu/docker-compose.yml

# provisioning
provisioning:
	ssh -t -i t2.pem ubuntu@216.47.98.209 '\
		echo "🧹 [1] 기존 Docker 관련 패키지 제거 중... (docker, containerd, runc 등 이미 설치된 항목 초기화)" && \
		sudo apt-get remove -y docker docker-engine docker.io containerd runc || true && \

		echo "🔄 [2] APT 패키지 목록 업데이트 중... (최신 패키지 정보 수신)" && \
		sudo apt-get update -y || true && \

		echo "📦 [3] Docker 설치를 위한 필수 패키지 설치 중... (HTTPS, 인증서, GPG 등)" && \
		sudo apt-get install -y apt-transport-https ca-certificates curl software-properties-common lsb-release gnupg || true && \

		echo "📁 [4] GPG 키 저장용 디렉터리 생성 중... (/usr/share/keyrings)" && \
		sudo mkdir -p /usr/share/keyrings || true && \

		echo "🔐 [5] Docker 공식 GPG 키 다운로드 및 등록 중... (패키지 인증용)" && \
		curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
			| sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg || true && \

		echo "📚 [6] Docker 공식 패키지 저장소 설정 중... (stable 채널 등록)" && \
		echo "deb [arch=$$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $$(lsb_release -cs) stable" \
			| sudo tee /etc/apt/sources.list.d/docker.list > /dev/null || true && \

		echo "🔁 [7] Docker 저장소 기준으로 APT 패키지 목록 재갱신 중..." && \
		sudo apt-get update -y || true && \

		echo "🐳 [8] Docker 본체 및 Compose 플러그인 설치 중... (도커, 빌드, 컴포즈 등)" && \
		sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || true && \

		echo "⚙️ [9] Docker 서비스 자동 시작(enable) 설정 중..." && \
		sudo systemctl enable docker || true && \

		echo "👤 [10] 현재 사용자를 docker 그룹에 추가 중... (sudo 없이 도커 사용 가능하게)" && \
		sudo usermod -aG docker ubuntu || true && \

		echo "✅ [11] Docker 버전 확인 중..." && \
		docker --version || true && \

		echo "✅ [12] Docker Compose (플러그인) 버전 확인 중..." && \
		docker compose version || true && \

		echo "🎉 [✓] Docker 및 Docker Compose 설치 완료! 로그아웃 후 다시 로그인해주세요." \
	'
