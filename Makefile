BIN     := clat
INSTALL := /usr/local/bin/$(BIN)

.PHONY: build release install uninstall clean

build:
	cargo build

release:
	cargo build --release

install: release
	cp target/release/$(BIN) $(INSTALL)
	@echo "Installed to $(INSTALL)"

uninstall:
	rm -f $(INSTALL)
	@echo "Removed $(INSTALL)"

clean:
	cargo clean
