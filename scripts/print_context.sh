#!/usr/bin/env bash

# Скрипт выводит пути и содержимое всех файлов в указанной директории или отдельного файла.
# Для каждого файла печатает строку вида // путь_к_файлу, а потом его содержимое.

set -e

TARGET="${1:-.}"

if [ -f "$TARGET" ]; then
  # Если передали одиночный файл
  echo "// $TARGET"
  cat "$TARGET"
  echo
elif [ -d "$TARGET" ]; then
  # Если передали директорию
  find "$TARGET" -type f \
      ! -path '*/.git/*' \
      ! -path '*/target/*' \
      ! -name '.DS_Store' \
      | sort | while read -r file; do
    echo "// $file"
    cat "$file"
    echo
  done
else
  echo "Error: $TARGET is neither a file nor a directory." >&2
  exit 1
fi
