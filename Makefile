run:
	cd axum && cargo run

clean:
	cd axum && cargo clean

release:
	cd axum && cargo build --release && ls -lh target/release/axum

run-release:
	./axum/target/release/axum
