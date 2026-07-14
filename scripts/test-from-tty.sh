#!/bin/sh
set -eu
printf 'Ejecuta este script desde una TTY real, no desde otra sesión gráfica.\n'
printf 'Sal de la sesión con Super+Shift+E o desde el menú de energía.\n'
exec /usr/local/bin/note-session
