BIN     := clat
INSTALL_DIR := $(HOME)/.clat
INSTALL := $(INSTALL_DIR)/$(BIN)

.PHONY: build release install uninstall clean

build:
	cargo build

release:
	cargo build --release

install: release
	mkdir -p $(INSTALL_DIR)
	cp target/release/$(BIN) $(INSTALL)
	@echo "Installed to $(INSTALL)"
	@if ! echo "$$PATH" | tr ':' '\n' | grep -qx "$(INSTALL_DIR)"; then \
		echo ""; \
		echo "Add ~/.clat to your PATH:"; \
		echo "  echo 'export PATH=\"\$$HOME/.clat:\$$PATH\"' >> ~/.zshrc  # zsh"; \
		echo "  echo 'export PATH=\"\$$HOME/.clat:\$$PATH\"' >> ~/.bashrc # bash"; \
	fi

uninstall:
	rm -f $(INSTALL)
	@echo "Removed $(INSTALL)"

clean:
	cargo clean
