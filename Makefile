all: converter index.html
converter: converter.rs
	rustc converter.rs -o converter
index.html: converter source
	./converter source
