# Changelog

## 0.2.1

- Corrige la detección de `greetd` en Debian, donde el daemon se instala en `/usr/sbin/greetd`.
- Evita depender de `command -v greetd` y del PATH del usuario.
- Hace `note-doctor` consistente con la ubicación real del daemon.
- Añade `scripts/preflight.sh` y un banner de versión para detectar copias mezcladas o antiguas.
