# 빌드 스테이지
FROM alpine AS builder

# 설정 파일 준비
RUN mkdir -p /tmp/fluent-bit/etc /tmp/fluent-bit/scripts /tmp/fluent-bit/log
COPY docker/fluent-bit/fluent-bit.conf /tmp/fluent-bit/etc/
COPY docker/fluent-bit/parser.conf /tmp/fluent-bit/etc/parsers.conf
COPY docker/fluent-bit/jsonb.lua /tmp/fluent-bit/scripts/

# 최종 스테이지
FROM fluent/fluent-bit:4.0.0-amd64

# 빌드 스테이지에서 파일 복사
COPY --from=builder /tmp/fluent-bit/etc/fluent-bit.conf /fluent-bit/etc/
COPY --from=builder /tmp/fluent-bit/etc/parsers.conf /fluent-bit/etc/parsers.conf
COPY --from=builder /tmp/fluent-bit/scripts/jsonb.lua /fluent-bit/scripts/
COPY --from=builder --chown=1000:1000 /tmp/fluent-bit/log /fluent-bit/log/

# 실행 설정
CMD ["/fluent-bit/bin/fluent-bit", "-c", "/fluent-bit/etc/fluent-bit.conf"]