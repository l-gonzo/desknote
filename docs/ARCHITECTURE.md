# Arquitectura del MVP

```text
greetd
  └─ cage
      └─ gtkgreet
          └─ note-session
              └─ dbus-run-session
                  └─ labwc (Wayland + XWayland)
                      ├─ note-shell (Rust + GTK4 layer shell)
                      ├─ swaybg
                      ├─ mako
                      ├─ NetworkManager applet
                      ├─ Blueman applet
                      └─ aplicaciones Wayland/XWayland
```

## Por qué labwc en la primera entrega

La interfaz, sesión, login, integración y empaquetado se pueden probar sin esperar a que el compositor Smithay propio tenga administración de ventanas, DMA-BUF, múltiples GPU, XWayland/XWM, portales y recuperación de errores suficientemente maduros. Labwc es una base Wayland pequeña que deliberadamente delega panel, fondo y demás piezas a clientes externos.

## Sustitución futura

`note-session-inner` es el único punto que necesita cambiar para reemplazar `labwc` por `note-compositor`. Note Shell ya usa `wlr-layer-shell`, por lo que puede conservarse con un compositor Smithay que implemente dicho protocolo.
