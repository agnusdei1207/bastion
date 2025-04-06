run-axum:
	cd axum && cargo run

clean-axum:
	cd axum && cargo clean

release-axum:
	cd axum && cargo build --release && ls -lh target/release/axum && ./target/release/axum

run-release:
	./axum/target/release/axum

push-axum:
	./docker/axum/push.sh

push-fluentd:
	./docker/fluentd/push.sh

push-suricata:
	./docker/suricata/push.sh


docker-compose-up:
	docker-compose -f docker/docker-compose.yml up -d
docker-compose-down:
	docker-compose -f docker/docker-compose.yml down