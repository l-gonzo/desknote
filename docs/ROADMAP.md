# Roadmap hacia el escritorio propio

## 0.1 — MVP actual

- Login gráfico con greetd, cage y gtkgreet.
- Sesión Wayland sin sesión Xorg.
- Compatibilidad X11 mediante XWayland.
- Barra y dock propios en Rust.
- Audio PipeWire, red NetworkManager, Bluetooth, portales, bloqueo y capturas.
- Instalador para Debian y Ubuntu mínimos.

## 0.2 — Shell utilizable diariamente

- Bandeja StatusNotifierItem real.
- Centro de control propio.
- Lanzador propio indexando archivos `.desktop`.
- Overview de ventanas y escritorios.
- Notificaciones y OSD propios.
- Configuración visual y persistencia TOML.

## 0.3 — `note-compositor` anidado con Smithay

- xdg-shell, layer-shell, seat, data device y output management.
- Ventanas flotantes, snapping, escritorios y atajos.
- Backend anidado para desarrollar desde Note Desktop.

## 0.4 — Backend DRM/KMS

- libseat/logind, udev, libinput, GBM/EGL.
- DMA-BUF feedback, modificadores y explicit sync.
- AMD/Intel Mesa y NVIDIA GBM.
- Múltiples monitores, VRR y escalado fraccional.

## 0.5 — XWayland/XWM y portales propios

- XWayland bajo demanda.
- Gestión unificada de ventanas Wayland y X11.
- Screencast PipeWire y RemoteDesktop.
- Recuperación del shell sin perder la sesión.
