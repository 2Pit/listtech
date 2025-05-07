#!/usr/bin/env bash

# Load environment variables from .env
export $(grep -v '^#' .env | xargs)

INDEXER_URL="http://localhost:$INDEXER_HTTP_PORT/v1/schema"

# JSON schema payload
read -r -d '' SCHEMA_JSON <<'EOF'
{
  "schema": {
    "name": "electronics_2",
    "columns": [
      { "name": "asin", "column_type": "string", "modifiers": ["id"] },
      { "name": "also_buy", "column_type": "string", "modifiers": ["nullable"] },
      { "name": "also_view", "column_type": "string", "modifiers": ["nullable"] },
      { "name": "brand_string", "column_type": "string", "modifiers": ["nullable"] },
      { "name": "image_url", "column_type": "string", "modifiers": ["nullable"] },
      { "name": "image_url_high_res", "column_type": "string", "modifiers": ["nullable"] },
      { "name": "price", "column_type": "double", "modifiers": ["nullable"] }
    ]
  }
}
EOF

# Send schema via HTTP POST
curl -v -X POST "$INDEXER_URL" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  --data-raw "$SCHEMA_JSON"
