# Roadmap

## 0.2 — Shell propio (esta entrega)

- Panel, dock y overview en Rust/GTK4.
- Centro de control.
- Configuración gráfica inicial.
- Cinco idiomas.
- Instalador Debian/Ubuntu más estricto.
- Paquete `.deb` local.
- Labwc como compositor temporal.

## 0.3 — Compositor anidado

- Workspace `note-compositor` con Smithay.
- Backend anidado dentro de una sesión existente.
- xdg-shell, layer-shell, input y ventanas básicas.
- Protocolo privado para el shell.
- XWayland rootless.
- Pruebas automatizadas de estado y geometría.

## 0.4 — Backend de hardware

- DRM/KMS, GBM y EGL.
- AMD, Intel y NVIDIA.
- Multi-GPU y PRIME.
- Varios monitores.
- Escalado entero y fraccional.
- DMA-BUF y sincronización explícita.
- Pixman como modo seguro.

## 0.5 — Experiencia de escritorio completa

- Escritorios dinámicos.
- Overview con miniaturas en vivo.
- Animaciones del compositor.
- Blur y sombras reales.
- Dock con agrupación, previews, badges y arrastrar/soltar.
- Gestos multitáctiles.
- Bloqueo propio mediante session-lock.
- Notificaciones y calendario propios.
- Accesibilidad inicial.

## 0.6 — Integración

- `xdg-desktop-portal-note`.
- Captura y screencast.
- Configuración nativa de pantallas.
- Gestión propia de red, Bluetooth, sonido y energía.
- Usuarios, fecha/hora, aplicaciones predeterminadas y privacidad.
- Actualizaciones y firmware.

## 0.7 — Distribución

- Repositorio APT firmado.
- COPR/repositorio RPM.
- PKGBUILD/AUR.
- CI para Debian, Ubuntu, Fedora y Arch.
- Pruebas en hardware AMD, Intel, NVIDIA y máquinas virtuales.
- Migraciones de configuración y rollback.

## 1.0

- Sesión diaria estable.
- Sin dependencia de labwc.
- Instalación y desinstalación limpia.
- Configuración básica sin terminal.
- Recuperación ante fallos de GPU o shell.
- Política documentada de compatibilidad y actualizaciones.
