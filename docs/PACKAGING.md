# Empaquetado y repositorios

## Paquetes planeados

La entrega alpha produce un paquete monolítico `note-desktop`. Antes de 1.0 debe dividirse:

```text
note-desktop              metapaquete
note-compositor           compositor Wayland
note-shell                panel, dock y overview
note-settings             configuración
note-session              sesión y servicios
note-greeter              acceso gráfico
note-portal               backend XDG Portal
note-themes               estilos e iconos
note-wallpapers           fondos
note-l10n-*               traducciones
```

## Debian/Ubuntu

Construcción local:

```bash
./scripts/build-deb.sh
sudo apt install ./dist/note-desktop_0.2.1_amd64.deb
```

Para distribución pública:

1. Compilar en un entorno limpio por arquitectura.
2. Firmar paquetes y metadatos.
3. Generar `Packages`, `Release`, `InRelease` y repositorio HTTPS.
4. Publicar un paquete pequeño `note-repository` que agregue la llave y la fuente.
5. Probar actualización, downgrade, purga y migración.

## Fedora/RPM

`packaging/rpm/note-desktop.spec` es un punto de partida. Hay que ajustar nombres de dependencias para la versión objetivo y publicar mediante COPR o repositorio propio.

## Arch

`packaging/arch/PKGBUILD` construye desde el árbol de fuentes. Para AUR debe usar una URL y checksum de release reales, y no un directorio local.

## Regla importante

El instalador Bash es una herramienta de bootstrap para desarrollo. La distribución final debe delegar dependencias, actualizaciones, migraciones y desinstalación al gestor de paquetes de cada sistema.
