#!/usr/bin/env bash
set -Eeuo pipefail

if [[ ${EUID} -eq 0 ]]; then
  sudo() {
    if [[ ${1:-} == "-v" ]]; then return 0; fi
    command "$@"
  }
else
  command -v sudo >/dev/null 2>&1 || {
    echo "Instala sudo o ejecuta este desinstalador como root." >&2
    exit 1
  }
fi

sudo -v
sudo systemctl disable --now greetd.service 2>/dev/null || true

latest_backup="$(sudo find /etc/greetd -maxdepth 1 -type f -name 'config.toml.note-backup-*' -printf '%T@ %p\n' 2>/dev/null | sort -nr | head -n1 | cut -d' ' -f2-)"
if [[ -n "$latest_backup" ]]; then
  echo "Restaurando $latest_backup"
  sudo cp -a "$latest_backup" /etc/greetd/config.toml
else
  sudo tee /etc/greetd/config.toml >/dev/null <<'CFG'
[terminal]
vt = 1
[default_session]
command = "agreety --cmd /bin/sh"
user = "greeter"
CFG
fi

sudo rm -f /usr/local/bin/note-{shell,session,launcher,terminal,files,browser,editor,network,audio,lock,logout,reboot,poweroff,screenshot,doctor,nvidia-kms,test-from-tty}
sudo rm -f /usr/local/libexec/note-{session-inner,autostart}
sudo rm -rf /usr/share/note-desktop /etc/xdg/note /etc/note-desktop
sudo rm -f /usr/share/wayland-sessions/note.desktop /etc/greetd/note-greet.css
sudo systemctl set-default multi-user.target

echo "Note Desktop fue retirado. Los paquetes del sistema se conservaron intencionalmente."
