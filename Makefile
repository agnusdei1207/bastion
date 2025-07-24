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
sqlx-prepare:
	cd axum && cargo sqlx prepare
push-axum:
	./docker/axum/push.sh
push-fluent-bit:
	./docker/fluent-bit/push.sh
push-suricata:
	./docker/suricata/push.sh
push-all:
	./docker/axum/push.sh
	./docker/fluent-bit/push.sh
	./docker/suricata/push.sh

# ssh
ssh:
	ssh -i k.pem ubuntu@216.47.98.91
scp:
	scp -i k.pem docker/docker-compose.yml ubuntu@216.47.98.91:/home/ubuntu/docker-compose.yml && \
scp -i k.pem script/provisioning.sh ubuntu@216.47.98.91:/home/ubuntu/provisioning.sh
provisioning: 
	ssh -i k.pem ubuntu@216.47.98.91 'bash /home/ubuntu/provisioning.sh'
restart:
	ssh -i k.pem ubuntu@216.47.98.91 'sudo docker compose down -v && \
	sudo docker rmi agnusdei1207/axum:latest agnusdei1207/fluent-bit:latest && \
	sudo docker compose -f /home/ubuntu/docker-compose.yml pull && \
	sudo docker compose -f /home/ubuntu/docker-compose.yml up -d && \
	sudo docker compose logs'
logs:
	ssh -i k.pem ubuntu@216.47.98.91 'sudo docker compose logs -f'