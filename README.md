# Note Desktop MVP

Una sesión de escritorio Wayland para Debian y Ubuntu mínimos, con estética sencilla inspirada en GNOME/macOS y shell propio escrito en Rust.

> Estado real: es un MVP instalable, no un entorno de escritorio terminado. Para que sea usable desde la primera prueba utiliza **labwc como compositor temporal**. La barra y el dock sí son componentes propios (`note-shell`). El objetivo posterior es sustituir labwc por `note-compositor` basado en Smithay.

## Incluye

- Login gráfico: greetd + cage + gtkgreet.
- Sesión exclusivamente Wayland.
- Aplicaciones X11 mediante XWayland rootless.
- Barra superior y dock propios en Rust/GTK4.
- Lanzador Wofi tematizado y notificaciones Mako tematizadas.
- Cuatro escritorios virtuales, snapping y atajos.
- PipeWire/WirePlumber, NetworkManager, BlueZ y UPower.
- Portales XDG para capturas y aplicaciones sandbox.
- Bloqueo con gtklock y capturas con grim/slurp.
- Diagnóstico de GPU, DRM y dependencias.
- Instalación y desinstalación reversibles.

## Sistemas objetivo

- Debian 13 o posterior, instalación mínima.
- Ubuntu Server 24.04/26.04 o posterior, sin escritorio.
- Arquitecturas que tengan los paquetes requeridos; amd64 es la ruta principal prevista.


## Qué está validado

- Sintaxis de todos los scripts Bash/POSIX.
- Parseo de los archivos TOML y XML.
- Estructura de instalación, respaldo y recuperación por TTY.
- APIs del shell contrastadas con GTK4 y gtk4-layer-shell actuales.

No pude compilar ni arrancar la sesión dentro del contenedor donde se generó el proyecto porque no tenía toolchain ni librerías gráficas y la instalación de paquetes agotó el tiempo disponible. Por eso debes tratar esta entrega como **alpha** y probar primero desde una TTY, no como sistema de producción.

## Instalación

Puedes ejecutarlo como usuario con `sudo` o directamente como `root` en una instalación mínima de Debian:

```bash
cd note-desktop-mvp
chmod +x scripts/*
./scripts/install.sh
```

Antes de reiniciar, prueba desde una TTY:

```bash
note-doctor
note-test-from-tty
```

Después:

```bash
sudo reboot
```

## Recuperación

Si la pantalla gráfica no inicia:

```text
Ctrl + Alt + F2
```

Inicia sesión y ejecuta:

```bash
sudo systemctl disable --now greetd
sudo systemctl set-default multi-user.target
```

## Atajos

| Atajo | Acción |
|---|---|
| Super + Espacio | Lanzador |
| Super + Enter | Terminal |
| Super + E | Archivos |
| Super + B | Navegador |
| Super + L | Bloquear |
| Super + Q | Cerrar ventana |
| Super + F | Pantalla completa |
| Super + M | Maximizar |
| Super + Flechas | Ajustar ventana |
| Super + Ctrl + ←/→ | Cambiar escritorio |
| Print | Captura de una región |

## NVIDIA

El MVP usa la ruta DRM/KMS + GBM de wlroots/labwc. Revisa:

```bash
cat /sys/module/nvidia_drm/parameters/modeset
note-doctor
```

Debe mostrar `Y` o `1`. Si ya instalaste el controlador propietario pero KMS aparece apagado:

```bash
sudo note-nvidia-kms
sudo reboot
```

El instalador no instala automáticamente el controlador propietario porque eso depende de los repositorios y de la GPU concreta. No necesitas una sesión Xorg; `xwayland` es suficiente para las aplicaciones X11.

En laptops híbridas existe `/etc/note-desktop/gpu.conf.example` para fijar `WLR_DRM_DEVICES` solamente cuando la detección automática no elija bien la GPU de presentación.

## Desinstalación

```bash
./scripts/uninstall.sh
```

Los paquetes APT instalados se conservan para no retirar dependencias compartidas por accidente.

## Corrección 0.1.2

Esta revisión sustituye `policykit-1-gnome` por `lxpolkit`. Debian 13 Trixie ya no ofrece el paquete anterior, mientras que `lxpolkit` está disponible tanto en Debian como en Ubuntu y se inicia explícitamente dentro de la sesión Note.

## Corrección 0.1.3

- Añade reintentos de APT.
- Desactiva temporalmente el pipelining HTTP/HTTPS durante la instalación.
- Instala primero el núcleo del escritorio y después las aplicaciones opcionales.
- Evita que un navegador o una dependencia opcional impida instalar `greetd`.
- Incluye `scripts/repair-apt.sh` para limpiar descargas parciales y reparar APT.
