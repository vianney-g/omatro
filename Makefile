PREFIX ?= $(HOME)/.local
OLD_PLUGIN := $(HOME)/.config/omarchy/plugins/io.github.vianney-g.omatro

.PHONY: test jokers-doc build run install uninstall

test:
	cargo test --workspace

# Regenerates docs/jokers.md from engine/src/joker.rs.
jokers-doc:
	UPDATE_JOKERS_DOC=1 cargo test -q -p omatro-engine joker_reference

build:
	cargo build --release -p omatro

run: build
	./target/release/omatro

# Installs into ~/.local: binary, launcher entry and icon.
install: build
	install -Dm755 target/release/omatro $(PREFIX)/bin/omatro
	install -Dm644 pkgbuild/omatro.desktop $(PREFIX)/share/applications/omatro.desktop
	install -Dm644 pkgbuild/omatro.svg $(PREFIX)/share/icons/hicolor/scalable/apps/omatro.svg

uninstall:
	rm -f $(PREFIX)/bin/omatro $(PREFIX)/share/applications/omatro.desktop \
	  $(PREFIX)/share/icons/hicolor/scalable/apps/omatro.svg
