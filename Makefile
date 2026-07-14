.PHONY: build check install uninstall archive

build:
	cargo build --release

check:
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings

install:
	./scripts/install.sh

uninstall:
	./scripts/uninstall.sh

archive:
	tar --exclude=target -czf note-desktop-mvp.tar.gz .
