#!/usr/bin/env bash
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "Usage: $0 <proto_file> <output_dir>"
  exit 1
fi

PROTO_FILE=$1
OUT_DIR=$2
THIRD_PARTY="corelib/proto/third_party"

mkdir -p "$OUT_DIR"

protoc \
  -I"$(dirname "$PROTO_FILE")" \
  -I"$THIRD_PARTY" \
  --openapiv2_out="$OUT_DIR" \
  --openapiv2_opt=logtostderr=true \
  "$PROTO_FILE"

echo "✅ Swagger JSON generated for $PROTO_FILE → $OUT_DIR"
