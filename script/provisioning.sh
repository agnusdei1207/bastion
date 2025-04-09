#!/bin/bash

set -e

echo "🔄 [1] APT 패키지 목록 업데이트 중..."
sudo apt-get update -y || true

echo "📦 [2] Docker 설치를 위한 필수 패키지 설치 중..."
sudo apt-get install -y apt-transport-https ca-certificates curl software-properties-common lsb-release gnupg || true

echo "📁 [3] GPG 키 디렉터리 생성 중..."
sudo mkdir -p /usr/share/keyrings || true

echo "🔐 [4] Docker GPG 키 등록 중..."
curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
  | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg || true

echo "📚 [5] Docker 저장소 추가 중..."
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" \
  | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null || true

echo "🔁 [6] Docker 저장소 기준으로 패키지 목록 재갱신 중..."
sudo apt-get update -y || true

echo "🐳 [7] Docker 본체 및 Compose 플러그인 설치 중..."
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || true

echo "⚙️ [8] Docker 서비스 자동 시작 설정 중..."
sudo systemctl enable docker || true

echo "👤 [9] docker 그룹에 사용자 추가 중..."
sudo usermod -aG docker $USER || true

echo "✅ [10] Docker 버전 확인..."
docker --version || true

echo "✅ [11] Docker Compose 버전 확인..."
docker compose version || true

echo "💾 [12] 스왑 상태 확인 중..."
sudo free -m
sudo swapon -s

echo "📂 [13] 스왑 파일 생성 중 (2GB)..."
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

echo "📝 [14] fstab에 스왑 자동 등록..."
echo "/swapfile swap swap defaults 0 0" | sudo tee -a /etc/fstab > /dev/null

echo "✅ [15] 스왑 활성화 확인..."
sudo swapon --show

echo "🔁 [16] systemd 재적용 중..."
sudo systemctl daemon-reexec

echo "🎉 [✓] Docker 및 Swap 설정 완료! 로그아웃 후 다시 로그인해야 docker 명령어가 sudo 없이 작동합니다."
