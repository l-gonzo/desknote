# Changelog

## 0.2.2

- Corrige la compilación con `gtk4-layer-shell 0.7.1`.
- Usa `set_margin`, `set_namespace(Some(...))` y `set_monitor(Some(...))`.
- Sustituye llamadas obsoletas a `hide()` por `set_visible(false)`.
- Convierte la consulta del buscador a `String` antes de reconstruir la cuadrícula.

## 0.2.1

- Corrige la detección de `greetd` en Debian, donde el daemon se instala en `/usr/sbin/greetd`.
- Evita depender de `command -v greetd` y del PATH del usuario.
- Hace `note-doctor` consistente con la ubicación real del daemon.
- Añade `scripts/preflight.sh` y un banner de versión para detectar copias mezcladas o antiguas.
