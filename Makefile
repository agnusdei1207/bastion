build:
	cd server && cargo build -o ../axum

clean:
	cd server && cargo clean

run:
	cd server && cargo run
