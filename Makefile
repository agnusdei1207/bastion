build:
	cd server && cargo build

clean:
	cd server && cargo clean

run:
	cd server && cargo run

release:
	cd server && cargo build --release

