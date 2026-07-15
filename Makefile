.PHONY: build check install uninstall package-deb archive

build:
	cargo build --release

check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

install: build
	sudo ./scripts/install-files.sh

uninstall:
	./scripts/uninstall.sh

package-deb:
	./scripts/build-deb.sh

archive:
	./scripts/make-release.sh
