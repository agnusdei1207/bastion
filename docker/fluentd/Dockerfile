# Fluentd 경량 이미지
FROM fluent/fluentd:v1.17.1-1.0

USER root

# HTTP 출력 플러그인만 설치
RUN gem install fluent-plugin-out-http --no-document

# Suricata 로그 디렉토리 생성 (외부 마운트면 생략 가능)
# RUN mkdir -p /var/log/suricata && chmod 755 /var/log/suricata

# Fluentd 설정 파일 복사
COPY docker/fluent/fluent.conf /fluentd/etc/fluent.conf

# Fluent 사용자로 전환
USER fluent

# Fluentd 실행
CMD ["fluentd", "-c", "/fluentd/etc/fluent.conf", "-p", "/fluentd/plugins"]
