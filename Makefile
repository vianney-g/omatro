PLUGIN_ID := io.github.vianney-g.omatro
PLUGIN_DIR := $(HOME)/.config/omarchy/plugins/$(PLUGIN_ID)

.PHONY: test plugin install

test:
	cd engine && cargo test

# Compile le moteur et le place à côté du QML du plugin.
plugin:
	cd engine && cargo build --release
	mkdir -p plugin/bin
	cp engine/target/release/omatro-engine plugin/bin/

# Relie le plugin au shell Omarchy (à faire une fois), puis l'active.
install: plugin
	ln -sfn $(CURDIR)/plugin $(PLUGIN_DIR)
	omarchy-shell shell rescanPlugins
	omarchy plugin enable $(PLUGIN_ID)
