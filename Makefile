run:
	cd server && cargo run

clean:
	cd server && cargo clean

release:
	cd server && cargo build --release

run-release:
	./server/target/release/server
