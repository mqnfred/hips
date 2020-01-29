all:
	cargo build --release
	strip target/release/hips
	mv target/release/hips hips
