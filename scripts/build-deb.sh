#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(awk -F'"' '/^version = / {print $2; exit}' "$ROOT_DIR/Cargo.toml")"
ARCH="$(dpkg --print-architecture)"
BUILD_DIR="$(mktemp -d)"
PACKAGE_ROOT="$BUILD_DIR/note-desktop"
OUTPUT_DIR="$ROOT_DIR/dist"
trap 'rm -rf "$BUILD_DIR"' EXIT

for command in cargo dpkg-deb dpkg; do
  command -v "$command" >/dev/null 2>&1 || {
    echo "Falta $command. Instala las herramientas de compilación y empaquetado." >&2
    exit 1
  }
done

pkg-config --exists gtk4 gtk4-layer-shell-0 || {
  echo "Faltan libgtk-4-dev o libgtk4-layer-shell-dev." >&2
  exit 1
}

cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release --locked 2>/dev/null || \
  cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release

mkdir -p "$PACKAGE_ROOT/DEBIAN" "$OUTPUT_DIR"
DESTDIR="$PACKAGE_ROOT" "$ROOT_DIR/scripts/install-files.sh" "$PACKAGE_ROOT"

INSTALLED_SIZE="$(du -sk "$PACKAGE_ROOT" | awk '{print $1}')"
cat > "$PACKAGE_ROOT/DEBIAN/control" <<CONTROL
Package: note-desktop
Version: $VERSION
Section: x11
Priority: optional
Architecture: $ARCH
Maintainer: Note Desktop Project <maintainers@invalid.example>
Installed-Size: $INSTALLED_SIZE
Depends: labwc, xwayland, greetd, cage, gtkgreet, wlrctl, dbus-user-session, xdg-desktop-portal, xdg-desktop-portal-wlr, xdg-desktop-portal-gtk, pipewire, wireplumber, network-manager, bluez, upower, polkitd, lxpolkit, mako-notifier, swaybg, foot, thunar, grim, slurp, wl-clipboard, libgtk-4-1, libgtk4-layer-shell0, libglib2.0-bin, locales
Recommends: pipewire-pulse, network-manager-gnome, blueman, pavucontrol, gtklock | swaylock, wdisplays, brightnessctl, playerctl, libnotify-bin, fonts-inter, fonts-cantarell, adwaita-icon-theme-full
Provides: x-window-manager
Description: experimental Wayland desktop with a GNOME-like workflow
 Note Desktop provides a custom Rust/GTK4 panel, dock, overview, control center,
 settings application and Wayland session. This alpha uses labwc as its
 temporary compositor while the Smithay compositor is developed.
CONTROL

cat > "$PACKAGE_ROOT/DEBIAN/postinst" <<'POSTINST'
#!/bin/sh
set -e
mkdir -p /var/lib/note-desktop /etc/greetd
if [ -f /etc/greetd/config.toml ] && [ ! -f /var/lib/note-desktop/greetd-config-pre-package.toml ]; then
  cp -a /etc/greetd/config.toml /var/lib/note-desktop/greetd-config-pre-package.toml
fi
if [ -f /etc/greetd/note-config.toml ]; then
  cp -f /etc/greetd/note-config.toml /etc/greetd/config.toml
fi
if command -v locale-gen >/dev/null 2>&1; then
  for locale in es_MX en_US pt_BR fr_FR de_DE; do
    sed -i -E "s/^# *(${locale}\.UTF-8 UTF-8)/\1/" /etc/locale.gen 2>/dev/null || true
  done
  locale-gen >/dev/null 2>&1 || true
fi
if id greeter >/dev/null 2>&1; then
  for group in video render input; do
    getent group "$group" >/dev/null 2>&1 && usermod -aG "$group" greeter || true
  done
fi
systemctl daemon-reload 2>/dev/null || true
systemctl enable NetworkManager.service 2>/dev/null || true
systemctl enable bluetooth.service 2>/dev/null || true
systemctl enable greetd.service 2>/dev/null || true
systemctl set-default graphical.target 2>/dev/null || true
update-desktop-database /usr/share/applications 2>/dev/null || true
gtk-update-icon-cache -q /usr/share/icons/hicolor 2>/dev/null || true
exit 0
POSTINST

cat > "$PACKAGE_ROOT/DEBIAN/prerm" <<'PRERM'
#!/bin/sh
set -e
if [ "$1" = remove ] || [ "$1" = deconfigure ]; then
  systemctl disable --now greetd.service 2>/dev/null || true
fi
exit 0
PRERM

cat > "$PACKAGE_ROOT/DEBIAN/postrm" <<'POSTRM'
#!/bin/sh
set -e
if [ "$1" = remove ] || [ "$1" = purge ]; then
  if [ -f /var/lib/note-desktop/greetd-config-pre-package.toml ]; then
    cp -f /var/lib/note-desktop/greetd-config-pre-package.toml /etc/greetd/config.toml
  fi
  systemctl daemon-reload 2>/dev/null || true
  update-desktop-database /usr/share/applications 2>/dev/null || true
  gtk-update-icon-cache -q /usr/share/icons/hicolor 2>/dev/null || true
fi
exit 0
POSTRM
chmod 0755 "$PACKAGE_ROOT/DEBIAN/postinst" "$PACKAGE_ROOT/DEBIAN/prerm" "$PACKAGE_ROOT/DEBIAN/postrm"

OUTPUT="$OUTPUT_DIR/note-desktop_${VERSION}_${ARCH}.deb"
dpkg-deb --root-owner-group --build "$PACKAGE_ROOT" "$OUTPUT"
echo "Paquete generado: $OUTPUT"
