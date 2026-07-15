#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DESTDIR="${1:-}"
PREFIX="${PREFIX:-/usr}"
ETC_DIR="${DESTDIR}/etc"
USR_DIR="${DESTDIR}${PREFIX}"

install -Dm0755 "$ROOT_DIR/target/release/note-shell" "$USR_DIR/bin/note-shell"
install -Dm0755 "$ROOT_DIR/target/release/note-settings" "$USR_DIR/bin/note-settings"

bin_scripts=(
  note-session note-overview note-control-center note-workspace note-terminal
  note-files note-browser note-editor note-lock note-logout note-reboot note-poweroff
  note-screenshot note-apply-settings note-doctor note-test-from-tty note-nvidia-kms
)
for script in "${bin_scripts[@]}"; do
  install -Dm0755 "$ROOT_DIR/scripts/$script" "$USR_DIR/bin/$script"
done

libexec_scripts=(note-session-inner note-greeter-session note-autostart)
for script in "${libexec_scripts[@]}"; do
  install -Dm0755 "$ROOT_DIR/scripts/$script" "$USR_DIR/libexec/note-desktop/$script"
done

install -Dm0644 "$ROOT_DIR/configs/wayland-session/note.desktop" "$USR_DIR/share/wayland-sessions/note.desktop"
install -Dm0644 "$ROOT_DIR/assets/note-settings.desktop" "$USR_DIR/share/applications/note-settings.desktop"
install -Dm0644 "$ROOT_DIR/assets/icons/note-desktop-symbolic.svg" "$USR_DIR/share/icons/hicolor/scalable/apps/note-desktop-symbolic.svg"
install -Dm0644 "$ROOT_DIR/assets/style/shell.css" "$USR_DIR/share/note-desktop/style/shell.css"
install -Dm0644 "$ROOT_DIR/assets/style/settings.css" "$USR_DIR/share/note-desktop/style/settings.css"
install -Dm0644 "$ROOT_DIR/assets/wallpapers/note-wave.svg" "$USR_DIR/share/note-desktop/wallpapers/note-wave.svg"
install -Dm0644 "$ROOT_DIR/assets/wallpapers/note-login.svg" "$USR_DIR/share/note-desktop/wallpapers/note-login.svg"
mkdir -p "$USR_DIR/share/themes/Note/labwc"
cp -a "$ROOT_DIR/assets/themes/Note/labwc/." "$USR_DIR/share/themes/Note/labwc/"
install -Dm0644 "$ROOT_DIR/configs/xdg-desktop-portal/note-portals.conf" "$USR_DIR/share/xdg-desktop-portal/note-portals.conf"

mkdir -p "$USR_DIR/share/note-desktop/locales"
cp -a "$ROOT_DIR/locales/." "$USR_DIR/share/note-desktop/locales/"

mkdir -p "$ETC_DIR/xdg/note/labwc"
for file in rc.xml menu.xml environment themerc-override; do
  install -Dm0644 "$ROOT_DIR/configs/labwc/$file" "$ETC_DIR/xdg/note/labwc/$file"
done
for file in autostart shutdown; do
  install -Dm0755 "$ROOT_DIR/configs/labwc/$file" "$ETC_DIR/xdg/note/labwc/$file"
done

install -Dm0644 "$ROOT_DIR/configs/greetd/config.toml" "$ETC_DIR/greetd/note-config.toml"
install -Dm0644 "$ROOT_DIR/configs/greetd/note-greet.css" "$ETC_DIR/greetd/note-greet.css"
install -Dm0644 "$ROOT_DIR/configs/gpu.conf.example" "$ETC_DIR/note-desktop/gpu.conf.example"

for unit in "$ROOT_DIR"/systemd/user/*.service "$ROOT_DIR"/systemd/user/*.target; do
  install -Dm0644 "$unit" "$USR_DIR/lib/systemd/user/$(basename "$unit")"
done

install -Dm0644 "$ROOT_DIR/README.md" "$USR_DIR/share/doc/note-desktop/README.md"
install -Dm0644 "$ROOT_DIR/LICENSE" "$USR_DIR/share/doc/note-desktop/LICENSE"
