.PHONY: install

PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
APPDIR := $(PREFIX)/share/applications
ICONDIR := $(PREFIX)/share/icons/hicolor/scalable/apps

install:
	cargo build --release
	install -Dm755 target/release/greatshot $(BINDIR)/greatshot
	install -Dm644 data/io.github.syed.greatshot.desktop $(APPDIR)/io.github.syed.greatshot.desktop
	install -Dm644 data/icons/hicolor/scalable/apps/io.github.syed.greatshot.svg $(ICONDIR)/io.github.syed.greatshot.svg
