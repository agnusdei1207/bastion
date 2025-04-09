FROM fluent/fluentd:v1.16-1

USER root

# 필요한 플러그인만 설치하여 이미지 크기 최소화
RUN apk add --no-cache --update --virtual .build-deps \
      sudo build-base ruby-dev \
    && gem install fluent-plugin-out-http fluent-plugin-parser --no-document \
    && apk del .build-deps \
    && rm -rf /var/cache/apk/*

# Suricata 로그 디렉토리 생성 및 권한 설정
RUN mkdir -p /var/log/suricata && chown -R fluent:fluent /var/log/suricata

# Fluentd 설정 파일 복사
COPY docker/fluentd/fluent.conf /fluentd/etc/fluent.conf

# 비루트 사용자로 전환하여 보안 강화
USER fluent

# Fluentd 실행
CMD ["fluentd", "-c", "/fluentd/etc/fluent.conf", "-p", "/fluentd/plugins"]
