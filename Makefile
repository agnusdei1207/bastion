# axum
run-axum:
	cd axum && cargo run
clean-axum:
	cd axum && cargo clean
release-axum:
	cd axum && cargo build --release && ls -lh target/release/axum && ./target/release/axum
run-release:
	./axum/target/release/axum

# docker
push-axum:
	./docker/axum/push.sh
push-fluentd:
	./docker/fluentd/push.sh
push-suricata:
	./docker/suricata/push.sh
push-all:
	./docker/axum/push.sh
	./docker/fluentd/push.sh
	./docker/suricata/push.sh

# docker-compose
docker-compose-up:
	docker-compose -f docker/docker-compose.yml up -d
docker-compose-down:
	docker-compose -f docker/docker-compose.yml down -v

# ssh
ssh:
	ssh -i test.pem ubuntu@216.47.98.207
scp:
	scp -i test.pem docker/docker-compose.yml ubuntu@216.47.98.207:/home/ubuntu/docker-compose.yml && \
scp -i test.pem script/provisioning.sh ubuntu@216.47.98.207:/home/ubuntu/provisioning.sh
provisioning: 
	ssh -i test.pem ubuntu@216.47.98.207 'bash /home/ubuntu/provisioning.sh'

