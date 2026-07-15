#!/usr/bin/env bash
set -Eeuo pipefail
export PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

fail=0
check() {
  local label="$1"; shift
  if "$@" >/dev/null 2>&1; then
    printf '[ok] %s\n' "$label"
  else
    printf '[falla] %s\n' "$label" >&2
    fail=1
  fi
}

check 'Debian/Ubuntu detectado' grep -Eq '^(ID|ID_LIKE)=.*(debian|ubuntu)' /etc/os-release
check 'APT funcional' apt-cache policy
check 'Paquete greetd instalado' dpkg-query -W -f='${Status}' greetd
if [[ -x /usr/sbin/greetd || -x /usr/bin/greetd ]]; then
  printf '[ok] ejecutable greetd localizado\n'
else
  printf '[falla] ejecutable greetd no localizado en /usr/sbin ni /usr/bin\n' >&2
  fail=1
fi
check 'systemd disponible' systemctl --version
check 'DRM disponible' test -d /dev/dri

exit "$fail"
