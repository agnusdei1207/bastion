run-axum:
	cd axum && cargo run

clean-axum:
	cd axum && cargo clean

release-axum:
	cd axum && cargo build --release && ls -lh target/release/axum && ./target/release/axum

run-release:
	./axum/target/release/axum
