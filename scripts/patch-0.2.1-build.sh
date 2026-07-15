#!/usr/bin/env bash
set -Eeuo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FILE="$ROOT_DIR/crates/note-shell/src/main.rs"
[[ -f "$FILE" ]] || { echo "No se encontró $FILE" >&2; exit 1; }
cp -a "$FILE" "$FILE.bak-$(date +%Y%m%d-%H%M%S)"
python3 - "$FILE" <<'PY2'
from pathlib import Path
import sys
p = Path(sys.argv[1])
s = p.read_text()
s = s.replace('.set_layer_shell_margin(', '.set_margin(')
s = s.replace('window.set_namespace("note-overview");', 'window.set_namespace(Some("note-overview"));')
s = s.replace('window.set_namespace("note-control-center");', 'window.set_namespace(Some("note-control-center"));')
s = s.replace('window.set_namespace(namespace);', 'window.set_namespace(Some(namespace));')
s = s.replace('window.set_monitor(monitor);', 'window.set_monitor(Some(monitor));')
s = s.replace(
    'rebuild_app_grid(&grid, &apps, entry.text().as_str(), &window);',
    'let query = entry.text().to_string();\n            rebuild_app_grid(&grid, &apps, &query, &window);',
)
s = s.replace('.hide();', '.set_visible(false);')
p.write_text(s)
PY2
printf 'Parche aplicado. Ahora ejecuta:\n  cargo clean\n  cargo build -p note-shell --release\n'
