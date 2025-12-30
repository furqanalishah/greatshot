.PHONY: install

PREFIX ?= $(HOME)/.local
BINDIR := $(PREFIX)/bin
APPDIR := $(PREFIX)/share/applications
ICONDIR := $(PREFIX)/share/icons/hicolor/scalable/apps

install:
	cargo build --release
	install -Dm755 target/release/greatshot $(BINDIR)/greatshot
	install -Dm644 data/io.github.syed.greatshot.desktop $(APPDIR)/io.github.syed.greatshot.desktop
	sed -i 's|^Exec=.*|Exec=$(BINDIR)/greatshot|' $(APPDIR)/io.github.syed.greatshot.desktop
	install -Dm644 data/icons/hicolor/scalable/apps/io.github.syed.greatshot.svg $(ICONDIR)/io.github.syed.greatshot.svg
	-update-desktop-database $(APPDIR)
	@test -f $(PREFIX)/share/icons/hicolor/index.theme || printf '%s\n' \
		'[Icon Theme]' \
		'Name=Hicolor' \
		'Comment=Fallback icon theme' \
		'Directories=scalable/apps' \
		'' \
		'[scalable/apps]' \
		'Size=128' \
		'Type=Scalable' \
		'MinSize=1' \
		'MaxSize=512' \
		'Context=Applications' \
		> $(PREFIX)/share/icons/hicolor/index.theme
	-gtk-update-icon-cache $(PREFIX)/share/icons/hicolor
