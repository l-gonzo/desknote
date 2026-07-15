# Note Desktop 0.2.0 Alpha

Note Desktop es un entorno Wayland experimental con flujo de trabajo inspirado en GNOME y una identidad visual inspirada en macOS. Esta entrega reemplaza la barra y el dock genéricos del MVP por un shell propio escrito en Rust y GTK4.

> Estado real: **alpha funcional**. El shell, overview, dock, centro de control y Configuración son propios. El compositor temporal sigue siendo **labwc**; `note-compositor` con Smithay forma parte de la siguiente etapa. No es todavía un reemplazo de GNOME 1.0.

## Qué contiene

- Sesión Wayland sin sesión Xorg.
- XWayland rootless administrado por labwc para aplicaciones antiguas.
- Pantalla de acceso con `greetd`, `cage` y `gtkgreet`.
- Barra superior propia con aplicación activa, reloj, calendario y estado.
- Dock flotante propio con favoritos, indicadores y enfoque de ventanas abiertas.
- Tema de ventanas Note con botones tipo semáforo colocados a la izquierda.
- Overview tipo GNOME con búsqueda, aplicaciones, ventanas y escritorios.
- Centro de control con Wi-Fi, Bluetooth, volumen, brillo y energía.
- Aplicación `note-settings` con apariencia, idioma y lanzadores de configuración.
- Cinco idiomas: español de México, inglés estadounidense, portugués de Brasil, francés y alemán.
- Temas claro/oscuro/automático, color de acento y opacidades configurables.
- PipeWire, WirePlumber, NetworkManager, BlueZ, PolicyKit y portales XDG.
- Modo de recuperación con renderizado por software para VirtualBox/VMware.
- Instalador directo para Debian 13 y Ubuntu reciente.
- Generador local de paquete `.deb` y esqueletos para RPM y Arch.

## Captura conceptual de la sesión

```text
┌──────────────────────────────────────────────────────────────────────┐
│ ●  Aplicación activa                 mar 14 jul 21:45       87%  ◉ │
└──────────────────────────────────────────────────────────────────────┘

                    Super abre el Overview

                ╭────────────────────────────╮
                │  Buscar aplicaciones…      │
                │  Ventanas | Aplicaciones   │
                │       Escritorios          │
                ╰────────────────────────────╯

                   ╭──────────────────────╮
                   │  Dock flotante       │
                   ╰──────────────────────╯
```

## Instalación en Debian o Ubuntu sin escritorio

Requisitos previos:

- Sistema Debian/Ubuntu ya instalado y con Internet.
- Usuario normal con `sudo`, o una sesión de root.
- Al menos 4 GB de almacenamiento libres durante la compilación.

```bash
unzip note-desktop-0.2.0.zip
cd note-desktop-0.2.0
chmod +x scripts/*
./scripts/install.sh
```

El instalador:

1. Detecta Debian o Ubuntu.
2. Habilita `universe` en Ubuntu cuando es necesario.
3. Comprueba candidatos APT sin depender del idioma del sistema.
4. Instala la pila gráfica, servicios, bibliotecas y compilador Rust.
5. Compila `note-shell` y `note-settings`.
6. Instala archivos de sesión, servicios y configuraciones.
7. Respalda la configuración anterior de greetd.
8. Agrega al usuario a `video`, `render` e `input` cuando existen.
9. Habilita el inicio gráfico.

### Prueba antes de reiniciar

Desde una TTY real, no desde otra sesión gráfica:

```bash
note-doctor
note-test-from-tty
```

Salir de la prueba:

```text
Super + Shift + E
```

Cuando la prueba funcione:

```bash
sudo reboot
```

### Recuperación ante pantalla negra

```text
Ctrl + Alt + F2
```

```bash
sudo systemctl disable --now greetd
sudo systemctl set-default multi-user.target
sudo reboot
```

## VirtualBox y VMware

Usa VMSVGA, 128 MB de vídeo y aceleración 3D. Si EGL falla durante los primeros segundos, `note-session-inner` vuelve a intentar automáticamente con Pixman.

También se puede forzar permanentemente en `/etc/note-desktop/gpu.conf`:

```bash
export WLR_RENDERER=pixman
export WLR_NO_HARDWARE_CURSORS=1
```

## NVIDIA

El instalador no instala automáticamente el controlador propietario. Primero instala el controlador adecuado de la distribución y comprueba DRM KMS. Después puedes ejecutar:

```bash
sudo note-nvidia-kms
sudo reboot
```

Diagnóstico:

```bash
note-doctor
cat /sys/module/nvidia_drm/parameters/modeset
```

## Atajos

| Atajo | Acción |
|---|---|
| `Super` o `Super + Espacio` | Overview |
| `Super + Enter` | Terminal |
| `Super + E` | Archivos |
| `Super + B` | Navegador |
| `Super + ,` | Configuración |
| `Super + L` | Bloquear |
| `Super + Q` | Cerrar ventana |
| `Super + M` | Minimizar |
| `Super + F` | Pantalla completa |
| `Super + ↑` | Maximizar |
| `Super + ←/→` | Ajustar a un lado |
| `Super + Ctrl + ←/→` | Cambiar escritorio |
| `Super + Ctrl + Shift + ←/→` | Mover ventana de escritorio |
| `Print` | Captura de región |
| `Super + Shift + E` | Cerrar sesión |

## Idiomas

El idioma se cambia en **Configuración → Apariencia → Idioma de la interfaz**. La elección se guarda en:

```text
~/.config/note-desktop/settings.toml
~/.config/environment.d/90-note-locale.conf
```

Idiomas incluidos:

```text
es-MX  en-US  pt-BR  fr-FR  de-DE
```

La aplicación se recarga al aplicar; algunas aplicaciones externas requieren cerrar sesión para recibir el nuevo locale.

## Compilar manualmente

En Debian 13:

```bash
sudo apt install build-essential pkg-config rustc cargo \
  libgtk-4-dev libgtk4-layer-shell-dev
cargo build --release
```

Las dependencias Rust están seleccionadas para ser compatibles con Rust 1.85 de Debian 13.

## Crear paquete `.deb`

```bash
chmod +x scripts/*
./scripts/build-deb.sh
```

Resultado:

```text
dist/note-desktop_0.2.0_<arquitectura>.deb
```

Instalación local:

```bash
sudo apt install ./dist/note-desktop_0.2.0_amd64.deb
```

Este `.deb` es para pruebas locales. Para poder ejecutar algún día `sudo apt install note-desktop`, todavía hace falta publicar y firmar un repositorio APT. Los esqueletos de distribución están en `packaging/`.

## Estructura

```text
crates/note-core       configuración, idiomas, aplicaciones y servicios
crates/note-shell      panel, dock, overview y centro de control
crates/note-settings   aplicación gráfica de configuración
configs/               labwc, greetd, sesión y portales
systemd/user/          servicios de la sesión
scripts/               instalación, sesión, diagnóstico y empaquetado
packaging/             Debian, Arch y RPM
assets/                estilos, iconos y fondos
locales/               traducciones
```

Consulta también:

- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/PACKAGING.md`

## Limitaciones conocidas

- Todavía no existe `note-compositor`; labwc sigue controlando ventanas y renderizado.
- No hay blur real del fondo: la transparencia actual se simula desde GTK.
- El overview enumera ventanas, pero no genera miniaturas vivas.
- La cantidad de escritorios es fija, no dinámica como GNOME.
- Algunas páginas de Configuración abren herramientas externas.
- El greeter todavía es gtkgreet tematizado, no un greeter propio.
- Los paquetes RPM y Arch son plantillas de mantenedor, no lanzamientos probados.

## Licencia

GNU GPL 3.0 o posterior. Consulta `LICENSE`.
