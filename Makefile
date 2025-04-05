run:
	cd server && cargo run

clean:
	cd server && cargo clean

release:
	cd server && cargo build --release && ls -lh server/target/release/server

run-release:
	./server/target/release/server
