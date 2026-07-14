#!/usr/bin/env bash
set -Eeuo pipefail

if [[ ${EUID} -eq 0 ]]; then
  sudo() {
    if [[ ${1:-} == "-v" ]]; then return 0; fi
    command "$@"
  }
else
  command -v sudo >/dev/null 2>&1 || {
    echo "Instala sudo o ejecuta este instalador como root." >&2
    exit 1
  }
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"

if [[ ! -r /etc/os-release ]]; then
  echo "No pude identificar la distribución." >&2
  exit 1
fi
# shellcheck disable=SC1091
source /etc/os-release
case "${ID:-}" in
  debian|ubuntu) ;;
  *) echo "Distribución no soportada por este instalador: ${ID:-desconocida}" >&2; exit 1 ;;
esac

sudo -v
sudo apt-get update

packages=(
  labwc xwayland greetd cage gtkgreet tuigreet
  dbus-user-session dbus-x11 xdg-utils
  xdg-desktop-portal xdg-desktop-portal-wlr xdg-desktop-portal-gtk
  pipewire pipewire-pulse wireplumber pavucontrol
  network-manager network-manager-gnome
  bluez blueman upower polkitd lxpolkit
  mako-notifier swaybg wofi foot thunar
  epiphany-browser gnome-text-editor eog evince file-roller gnome-calculator
  gtklock grim slurp wl-clipboard brightnessctl playerctl libnotify-bin
  libgtk-4-dev libgtk4-layer-shell-dev libglib2.0-dev pkg-config build-essential
  libgl1-mesa-dri mesa-vulkan-drivers mesa-utils vulkan-tools
  curl ca-certificates git pciutils adwaita-icon-theme-full fonts-cantarell fonts-inter
)

available=()
for package in "${packages[@]}"; do
  candidate="$(apt-cache policy "$package" 2>/dev/null | awk '/Candidate:/ {print $2; exit}')"
  if [[ -n "$candidate" && "$candidate" != "(none)" ]]; then
    available+=("$package")
  else
    echo "[aviso] Paquete sin candidato instalable, se omite: $package"
  fi
done
sudo env DEBIAN_FRONTEND=noninteractive apt-get install -y "${available[@]}"

critical=(labwc Xwayland greetd cage gtkgreet lxpolkit pkg-config)
for command in "${critical[@]}"; do
  command -v "$command" >/dev/null 2>&1 || { echo "Falta dependencia crítica: $command" >&2; exit 1; }
done
pkg-config --exists gtk4 || { echo "No se encontró gtk4 mediante pkg-config" >&2; exit 1; }
pkg-config --exists gtk4-layer-shell-0 || { echo "No se encontró gtk4-layer-shell-0" >&2; exit 1; }

if ! command -v cargo >/dev/null 2>&1 || ! command -v rustup >/dev/null 2>&1; then
  echo "Instalando Rust estable con rustup para compilar Note Shell..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
fi
rustup toolchain install stable --profile minimal
cargo +stable build --manifest-path "$ROOT_DIR/Cargo.toml" --release

sudo install -Dm0755 "$ROOT_DIR/target/release/note-shell" /usr/local/bin/note-shell

for script in note-session note-launcher note-terminal note-files note-browser note-editor note-network note-audio note-lock note-logout note-reboot note-poweroff note-screenshot note-doctor note-nvidia-kms; do
  sudo install -Dm0755 "$ROOT_DIR/scripts/$script" "/usr/local/bin/$script"
done
for script in note-session-inner note-autostart; do
  sudo install -Dm0755 "$ROOT_DIR/scripts/$script" "/usr/local/libexec/$script"
done
sudo install -Dm0755 "$ROOT_DIR/scripts/test-from-tty.sh" /usr/local/bin/note-test-from-tty

sudo install -Dm0644 "$ROOT_DIR/assets/note-wallpaper.svg" /usr/share/note-desktop/wallpapers/note-wallpaper.svg
sudo install -Dm0644 "$ROOT_DIR/configs/gpu.conf.example" /etc/note-desktop/gpu.conf.example
sudo install -Dm0644 "$ROOT_DIR/configs/wayland-session/note.desktop" /usr/share/wayland-sessions/note.desktop

sudo mkdir -p /etc/xdg/note/labwc
for file in rc.xml menu.xml environment themerc-override; do
  sudo install -Dm0644 "$ROOT_DIR/configs/labwc/$file" "/etc/xdg/note/labwc/$file"
done
for file in autostart shutdown; do
  sudo install -Dm0755 "$ROOT_DIR/configs/labwc/$file" "/etc/xdg/note/labwc/$file"
done
sudo install -Dm0644 "$ROOT_DIR/configs/wofi/config" /etc/xdg/note/wofi/config
sudo install -Dm0644 "$ROOT_DIR/configs/wofi/style.css" /etc/xdg/note/wofi/style.css
sudo install -Dm0644 "$ROOT_DIR/configs/mako/config" /etc/xdg/note/mako/config

sudo mkdir -p /etc/greetd
if [[ -f /etc/greetd/config.toml ]]; then
  sudo cp -a /etc/greetd/config.toml "/etc/greetd/config.toml.note-backup-$STAMP"
fi
sudo install -Dm0644 "$ROOT_DIR/configs/greetd/config.toml" /etc/greetd/config.toml
sudo install -Dm0644 "$ROOT_DIR/configs/greetd/note-greet.css" /etc/greetd/note-greet.css

if id greeter >/dev/null 2>&1; then
  for group in video render input; do
    getent group "$group" >/dev/null 2>&1 && sudo usermod -aG "$group" greeter || true
  done
fi

sudo systemctl enable NetworkManager.service 2>/dev/null || true
sudo systemctl enable bluetooth.service 2>/dev/null || true
sudo systemctl enable greetd.service
sudo systemctl set-default graphical.target

cat <<'MSG'

Note Desktop MVP quedó instalado.

Prueba recomendada antes de reiniciar:
  1. Cambia a una TTY con Ctrl+Alt+F3.
  2. Inicia sesión.
  3. Ejecuta: note-doctor
  4. Ejecuta: note-test-from-tty

Cuando confirmes que abre, reinicia para ver el login gráfico:
  sudo reboot

Recuperación si la pantalla queda negra:
  Ctrl+Alt+F2
  sudo systemctl disable --now greetd

Este MVP usa labwc como compositor temporal. La barra y el dock son Note Shell en Rust.
MSG
