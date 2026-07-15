# Arquitectura de Note Desktop

## Capas actuales

```text
greetd
└── cage
    └── gtkgreet
        └── note-session
            └── labwc
                ├── XWayland
                ├── note-shell
                ├── mako
                ├── lxpolkit
                ├── swaybg
                └── aplicaciones
```

`note-shell` es una aplicación GTK4 con superficies layer-shell separadas:

- panel por monitor;
- dock por monitor;
- overview overlay;
- centro de control overlay.

`note-core` concentra configuración, traducciones, descubrimiento de archivos `.desktop`, llamadas a `wlrctl`, audio, brillo, Wi-Fi y Bluetooth.

`note-settings` escribe configuración de usuario y pide al shell y a labwc que se recarguen.

## Servicios de usuario

La sesión usa `note-session.target`:

```text
note-session.target
├── note-shell.service
├── note-notifications.service
└── note-polkit-agent.service
```

El autostart de labwc importa el entorno Wayland y activa el target. Esto evita lanzar todo desde un único script monolítico.

## Configuración

Configuración global:

```text
/etc/xdg/note/labwc/
/etc/greetd/
/etc/note-desktop/
```

Configuración de usuario:

```text
~/.config/note-desktop/settings.toml
~/.config/labwc/rc.xml
~/.config/environment.d/90-note-locale.conf
```

## IPC actual

- `gapplication action mx.note.desktop.shell overview`
- `gapplication action mx.note.desktop.shell control-center`
- `gapplication action mx.note.desktop.shell reload`
- `wlrctl` para enumerar y enfocar ventanas.
- D-Bus de NetworkManager, PipeWire y demás a través de sus herramientas de usuario.

## Arquitectura objetivo

```text
note-greeter
└── note-session
    └── note-compositor (Smithay)
        ├── protocolo privado note-shell-v1
        ├── XWayland bajo demanda
        ├── note-shell
        ├── note-settings-daemon
        ├── note-notifications
        ├── note-lock
        └── xdg-desktop-portal-note
```

El compositor futuro absorberá:

- DRM/KMS, GBM y EGL;
- GPU múltiple;
- sincronización explícita;
- DMA-BUF feedback;
- gestión de ventanas;
- escritorios dinámicos;
- miniaturas y animaciones;
- blur, sombras y redondeo real;
- gestos y direct scanout;
- XWayland on demand.

El shell permanecerá separado para que una caída del dock o del panel no mate las aplicaciones.
