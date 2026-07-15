#!/usr/bin/env bash
set -Eeuo pipefail
if [[ ${EUID} -ne 0 ]]; then exec sudo "$0" "$@"; fi

systemctl disable --now greetd.service 2>/dev/null || true
systemctl set-default multi-user.target || true

latest_backup="$(ls -1t /var/lib/note-desktop/greetd-config-*.toml 2>/dev/null | head -n1 || true)"
if [[ -n "$latest_backup" ]]; then
  install -Dm0644 "$latest_backup" /etc/greetd/config.toml
else
  rm -f /etc/greetd/config.toml
fi

rm -f \
  /usr/bin/note-shell /usr/bin/note-settings /usr/bin/note-session \
  /usr/bin/note-overview /usr/bin/note-control-center /usr/bin/note-workspace \
  /usr/bin/note-terminal /usr/bin/note-files /usr/bin/note-browser /usr/bin/note-editor \
  /usr/bin/note-lock /usr/bin/note-logout /usr/bin/note-reboot /usr/bin/note-poweroff \
  /usr/bin/note-screenshot /usr/bin/note-apply-settings /usr/bin/note-doctor \
  /usr/bin/note-test-from-tty /usr/bin/note-nvidia-kms
rm -rf /usr/libexec/note-desktop /usr/share/note-desktop /usr/share/themes/Note /etc/xdg/note /etc/note-desktop
rm -f /usr/share/wayland-sessions/note.desktop /usr/share/applications/note-settings.desktop
rm -f /usr/share/icons/hicolor/scalable/apps/note-desktop-symbolic.svg
rm -f /usr/share/xdg-desktop-portal/note-portals.conf
rm -f /usr/lib/systemd/user/note-shell.service /usr/lib/systemd/user/note-notifications.service
rm -f /usr/lib/systemd/user/note-polkit-agent.service /usr/lib/systemd/user/note-session.target
rm -f /etc/greetd/note-config.toml /etc/greetd/note-greet.css

systemctl daemon-reload
update-desktop-database /usr/share/applications 2>/dev/null || true
gtk-update-icon-cache -q /usr/share/icons/hicolor 2>/dev/null || true

echo "Note Desktop eliminado. Los paquetes compartidos se conservaron."
