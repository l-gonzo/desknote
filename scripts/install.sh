#!/usr/bin/env bash
set -Eeuo pipefail
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"
VERSION="$(cat "$ROOT_DIR/VERSION" 2>/dev/null || printf 'desconocida')"
printf 'Note Desktop installer %s\n' "$VERSION"

if [[ ! -r /etc/os-release ]]; then
  echo "No se pudo identificar la distribución." >&2
  exit 1
fi
# shellcheck disable=SC1091
source /etc/os-release
case "${ID:-}" in
  debian|ubuntu) ;;
  *) echo "Este instalador directo soporta Debian y Ubuntu. Usa packaging/ para otras distribuciones." >&2; exit 1 ;;
esac

if [[ ${EUID} -eq 0 ]]; then
  SUDO=()
  BUILD_USER="${SUDO_USER:-root}"
else
  command -v sudo >/dev/null 2>&1 || { echo "Instala sudo o ejecuta como root." >&2; exit 1; }
  sudo -v
  SUDO=(sudo)
  BUILD_USER="$(id -un)"
fi

APT_OPTS=(
  -o Acquire::Retries=5
  -o Acquire::http::Pipeline-Depth=0
  -o Acquire::https::Pipeline-Depth=0
  -o Acquire::Queue-Mode=access
)

apt_run() {
  local attempt
  for attempt in 1 2 3; do
    if "${SUDO[@]}" env DEBIAN_FRONTEND=noninteractive apt-get "${APT_OPTS[@]}" "$@"; then
      return 0
    fi
    echo "[aviso] APT falló ($attempt/3); limpiando descargas parciales." >&2
    "${SUDO[@]}" rm -f /var/cache/apt/archives/partial/* 2>/dev/null || true
    sleep 3
  done
  return 1
}

candidate() {
  LC_ALL=C apt-cache policy "$1" 2>/dev/null | awk '/^[[:space:]]*Candidate:/ {print $2; exit}'
}

filter_available() {
  local package value
  for package in "$@"; do
    value="$(candidate "$package")"
    if [[ -n "$value" && "$value" != "(none)" ]]; then
      printf '%s\n' "$package"
    else
      echo "[aviso] Sin candidato instalable: $package" >&2
    fi
  done
}

apt_run update
if [[ ${ID} == ubuntu ]]; then
  apt_run install -y software-properties-common
  "${SUDO[@]}" add-apt-repository -y universe || true
  apt_run update
fi

core_packages=(
  labwc xwayland greetd cage gtkgreet wlrctl
  dbus-user-session dbus-x11 libglib2.0-bin xdg-utils
  xdg-desktop-portal xdg-desktop-portal-wlr xdg-desktop-portal-gtk
  pipewire pipewire-pulse wireplumber
  network-manager network-manager-gnome
  bluez upower polkitd lxpolkit
  mako-notifier swaybg foot thunar
  grim slurp wl-clipboard brightnessctl playerctl libnotify-bin
  libgtk-4-dev libgtk4-layer-shell-dev libglib2.0-dev pkg-config build-essential
  rustc cargo python3 locales
  libgl1-mesa-dri mesa-vulkan-drivers mesa-utils vulkan-tools
  curl ca-certificates git pciutils adwaita-icon-theme-full fonts-cantarell fonts-inter
)
optional_packages=(
  gtklock swaylock wdisplays pavucontrol blueman
  epiphany-browser firefox-esr gnome-text-editor eog evince file-roller gnome-calculator
)

mapfile -t core_available < <(filter_available "${core_packages[@]}")
mapfile -t optional_available < <(filter_available "${optional_packages[@]}")
apt_run install -y "${core_available[@]}"

if command -v locale-gen >/dev/null 2>&1; then
  for locale in es_MX en_US pt_BR fr_FR de_DE; do
    "${SUDO[@]}" sed -i -E "s/^# *(${locale}\.UTF-8 UTF-8)/\1/" /etc/locale.gen 2>/dev/null || true
  done
  "${SUDO[@]}" locale-gen >/dev/null || true
fi

for package in "${optional_available[@]}"; do
  apt_run install -y "$package" || echo "[aviso] Paquete opcional omitido: $package" >&2
done

critical_commands=(labwc Xwayland cage gtkgreet wlrctl pkg-config cargo rustc)
for command in "${critical_commands[@]}"; do
  command -v "$command" >/dev/null 2>&1 || { echo "Falta dependencia crítica: $command" >&2; exit 1; }
done

# En Debian greetd es un daemon de sbin. No debe validarse sólo con command -v,
# porque el PATH de usuarios normales puede omitir /usr/sbin.
greetd_bin=""
for candidate_path in /usr/sbin/greetd /usr/bin/greetd; do
  if [[ -x "$candidate_path" ]]; then
    greetd_bin="$candidate_path"
    break
  fi
done
if [[ -z "$greetd_bin" ]] && command -v greetd >/dev/null 2>&1; then
  greetd_bin="$(command -v greetd)"
fi
if [[ -z "$greetd_bin" ]]; then
  echo "Falta greetd: el paquete no dejó un ejecutable utilizable." >&2
  echo "Diagnóstico: dpkg-query -W greetd; dpkg -L greetd" >&2
  exit 1
fi
printf '[ok] greetd: %s\n' "$greetd_bin"
pkg-config --exists gtk4 || { echo "No se encontró GTK4 con pkg-config." >&2; exit 1; }
pkg-config --exists gtk4-layer-shell-0 || { echo "No se encontró gtk4-layer-shell." >&2; exit 1; }

build_command=(cargo build --manifest-path "$ROOT_DIR/Cargo.toml" --release)
if [[ ${EUID} -eq 0 && ${BUILD_USER} != root ]]; then
  BUILD_HOME="$(getent passwd "$BUILD_USER" | cut -d: -f6)"
  sudo -u "$BUILD_USER" env HOME="$BUILD_HOME" "${build_command[@]}"
else
  "${build_command[@]}"
fi

"${SUDO[@]}" "$ROOT_DIR/scripts/install-files.sh"

"${SUDO[@]}" mkdir -p /var/lib/note-desktop /etc/greetd
if [[ -f /etc/greetd/config.toml ]]; then
  "${SUDO[@]}" cp -a /etc/greetd/config.toml "/var/lib/note-desktop/greetd-config-$STAMP.toml"
fi
"${SUDO[@]}" install -Dm0644 /etc/greetd/note-config.toml /etc/greetd/config.toml

if id greeter >/dev/null 2>&1; then
  for group in video render input; do
    getent group "$group" >/dev/null 2>&1 && "${SUDO[@]}" usermod -aG "$group" greeter || true
  done
fi
if [[ ${BUILD_USER} != root ]] && id "$BUILD_USER" >/dev/null 2>&1; then
  for group in video render input; do
    getent group "$group" >/dev/null 2>&1 && "${SUDO[@]}" usermod -aG "$group" "$BUILD_USER" || true
  done
fi

"${SUDO[@]}" systemctl daemon-reload
"${SUDO[@]}" systemctl enable NetworkManager.service 2>/dev/null || true
"${SUDO[@]}" systemctl enable bluetooth.service 2>/dev/null || true
"${SUDO[@]}" systemctl enable greetd.service
"${SUDO[@]}" systemctl set-default graphical.target
"${SUDO[@]}" update-desktop-database /usr/share/applications 2>/dev/null || true
"${SUDO[@]}" gtk-update-icon-cache -q /usr/share/icons/hicolor 2>/dev/null || true

cat <<'MSG'

Note Desktop 0.2.2 quedó instalado.

Prueba antes de reiniciar:
  1. Ctrl+Alt+F3
  2. Inicia sesión
  3. note-doctor
  4. note-test-from-tty

Salir de la prueba: Super+Shift+E o ejecuta note-logout.
Cuando confirme que inicia: sudo reboot

Recuperación:
  Ctrl+Alt+F2
  sudo systemctl disable --now greetd
  sudo systemctl set-default multi-user.target

Esta versión ya incluye shell, overview, dock, centro de control, configuración
e idiomas. El compositor sigue siendo labwc mientras note-compositor se desarrolla.
MSG
