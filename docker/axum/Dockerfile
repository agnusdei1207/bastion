FROM alpine:latest

RUN apk add --no-cache ca-certificates

COPY ./axum/target/release/axum /app/axum

RUN chmod +x /app/axum

ENTRYPOINT ["/app/axum"]
