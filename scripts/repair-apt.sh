#!/usr/bin/env bash
set -Eeuo pipefail

if [[ ${EUID} -eq 0 ]]; then
  sudo() { command "$@"; }
elif ! command -v sudo >/dev/null 2>&1; then
  echo "Ejecuta este script como root o instala sudo." >&2
  exit 1
fi

APT_OPTS=(
  -o Acquire::Retries=5
  -o Acquire::http::Pipeline-Depth=0
  -o Acquire::https::Pipeline-Depth=0
  -o Acquire::Queue-Mode=access
)

echo "[1/5] Terminando configuraciones pendientes de dpkg..."
sudo dpkg --configure -a

echo "[2/5] Limpiando descargas parciales de APT..."
sudo rm -f /var/cache/apt/archives/partial/* 2>/dev/null || true
sudo apt-get clean

echo "[3/5] Reparando dependencias incompletas..."
sudo env DEBIAN_FRONTEND=noninteractive apt-get "${APT_OPTS[@]}" -f install -y

echo "[4/5] Actualizando índices con reintentos y sin pipelining HTTP..."
sudo apt-get "${APT_OPTS[@]}" update

echo "[5/5] Probando la descarga que detonó el fallo..."
sudo env DEBIAN_FRONTEND=noninteractive apt-get "${APT_OPTS[@]}" install -y libflite1

echo
printf '%s\n' "APT quedó reparado. Vuelve a ejecutar: ./scripts/install.sh"
