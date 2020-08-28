all: converter/target/release/converter index.html
converter/target/release/converter: converter/src/main.rs
	cd converter; cargo build --release
index.html: converter/target/release/converter source
	converter/target/release/converter source
