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

docker-run:
	docker run -p 3000:3000 --rm --name agnusdei1207/axum:latest