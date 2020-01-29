all:
	cargo build --release
	cp target/release/hips hips
	strip hips
