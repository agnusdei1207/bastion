#!/bin/bash

# 변수 설정
IMAGE_NAME="axum"
TAG="latest"
DOCKER_USERNAME="agnusdei1207"
DOCKER_IMAGE="$DOCKER_USERNAME/$IMAGE_NAME:$TAG"
DOCKERFILE="docker/axum/Dockerfile"

source docker/common/common.sh

# 현재 OS 확인
OS=$(uname -s)
echo "🖥️ 현재 OS: $OS"

# 이전 빌드된 이미지가 있다면 삭제
echo "🗑️ 이전 이미지 삭제 중..."
docker rmi -f $DOCKER_IMAGE

# Dockerfile 존재 여부 체크
if [ ! -f $DOCKERFILE ]; then
    echo "❌ $DOCKERFILE 파일을 찾을 수 없습니다."
    exit 1
fi

# Docker 이미지 빌드
echo "🔨 이미지 빌드 중..."
docker build --progress=auto --platform linux/amd64 -t $DOCKER_IMAGE -f $DOCKERFILE . --no-cache

# 이미지가 정상적으로 빌드됐는지 확인
if ! docker image inspect $DOCKER_IMAGE > /dev/null 2>&1; then
    echo "❌ 빌드된 이미지가 존재하지 않습니다: $DOCKER_IMAGE"
    exit 1
fi

# 빌드된 이미지 푸시
echo "📤 이미지 푸시 중..."
docker push $DOCKER_IMAGE

# 푸시 완료 후 메시지 출력
echo "✅ 이미지가 성공적으로 푸시되었습니다: $DOCKER_IMAGE"
