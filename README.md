# GreatShot

Minimal, fast screenshot + annotation tool for Linux.

Motivation: Flameshot isn’t reliable on GNOME Wayland, so GreatShot focuses on a clean, portal-based workflow that works on modern Fedora/GNOME setups.

## Build (host)
Fedora dependencies:
```
sudo dnf install rust cargo gtk4-devel libadwaita-devel pkgconf-pkg-config gcc
```

Build and run:
```
cargo run
```
