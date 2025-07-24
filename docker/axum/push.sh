#!/bin/bash

# 변수 설정
IMAGE_NAME="axum"
TAG="latest"
DOCKER_USERNAME="agnusdei1207"
DOCKER_IMAGE="$DOCKER_USERNAME/$IMAGE_NAME:$TAG"
DOCKERFILE="docker/axum/Dockerfile"

source docker/common/common.sh


