FROM fluent/fluent-bit:latest

USER root

# Lua JSON 모듈 설치용 luarocks 추가
RUN apk add --no-cache lua5.1 lua5.1-dev luarocks build-base \
    && luarocks install dkjson \
    && apk del build-base

# 설정 파일 복사
COPY docker/fluent-bit/fluent-bit.conf /fluent-bit/etc/fluent-bit.conf
COPY docker/fluent-bit/parser.conf /fluent-bit/etc/parsers.conf
COPY docker/fluent-bit/jsonb.lua /fluent-bit/scripts/jsonb.lua

# 로그 디렉토리 생성 및 권한 설정
RUN mkdir -p /fluent-bit/log && \
    chown -R fluent:fluent /fluent-bit/log

USER fluent

# 설정 파일 경로 지정하여 실행
CMD ["/fluent-bit/bin/fluent-bit", "-c", "/fluent-bit/etc/fluent-bit.conf"]