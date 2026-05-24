.PHONY: build install clean test

build:
	cargo build --release -p fountain-lsp-core

install: build
	sudo cp target/release/fountain-lsp-core /usr/local/bin/fountain-lsp

clean:
	cargo clean

test:
	cd ../fountain-tree-sitter && node test/parser.test.js