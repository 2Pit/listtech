#!/usr/bin/env bash

# Скрипт выводит пути и содержимое всех неигнорируемых файлов внутри указанной директории
# или одного файла, если передан файл.
# Пути печатаются с двумя слешами // file_path

set -e

TARGET="${1:-.}"

if [ -f "$TARGET" ]; then
  echo "// $TARGET"
  cat "$TARGET"
  echo
elif [ -d "$TARGET" ]; then
  git -C "$TARGET" ls-files --full-name | while read -r file; do
    echo "// $TARGET/$file"
    cat "$file"
    echo
  done
else
  echo "Error: $TARGET is neither a file nor a directory." >&2
  exit 1
fi
