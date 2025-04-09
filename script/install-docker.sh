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

echo "🐳 [7] Docker 및 Compose 설치 중..."
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin || true

echo "⚙️ [8] Docker 서비스 자동 시작 설정 중..."
sudo systemctl enable docker || true

echo "👤 [9] docker 그룹에 사용자 추가 중..."
sudo usermod -aG docker $USER || true

echo "✅ [10] Docker 버전 확인..."
docker --version || true

echo "✅ [11] Docker Compose 버전 확인..."
docker compose version || true
