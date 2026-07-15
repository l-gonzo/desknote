# Validación de esta entrega

Comprobaciones realizadas en el entorno de generación:

- Sintaxis de todos los scripts Bash/POSIX shell.
- Parseo de todos los archivos TOML.
- Parseo XML de configuraciones labwc y recursos SVG.
- Correspondencia de claves en los cinco catálogos de traducción.
- Instalación simulada mediante `scripts/install-files.sh` dentro de un `DESTDIR` temporal.
- Revisión de compatibilidad de dependencias: GTK4 Rust 0.10.x y gtk4-layer-shell 0.7.x para Rust 1.83 o posterior.

No se pudo ejecutar `cargo build` en el contenedor porque no tenía Rust/GTK instalados y el acceso DNS del contenedor a los repositorios estaba bloqueado. El workflow de CI incluido compila el workspace dentro de Debian 13 cuando se publique en un repositorio Git.

Antes de usarlo fuera de una máquina virtual se recomienda:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
```
