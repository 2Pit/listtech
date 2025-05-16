#!/usr/bin/env bash

# Load environment variables from .env
export $(grep -v '^#' .env | xargs)

INDEXER_URL="http://localhost:$INDEXER_HTTP_PORT/v1/schema"

# JSON schema payload
read -r -d '' SCHEMA_JSON <<'EOF'
{
  "schema": {
    "name": "electronics",
    "columns": [
      { "name": "asin", "column_type": "text", "modifiers": ["id"] },
      { "name": "price", "column_type": "double", "modifiers": ["equals", "fast_sortable"] },
      { "name": "title", "column_type": "text", "modifiers": ["equals", "full_text"] },
      { "name": "main_cat", "column_type": "text", "modifiers": ["equals", "full_text"] },
      { "name": "description", "column_type": "text", "modifiers": ["full_text"] },
      { "name": "timestamp_creation_ms", "column_type": "date_time", "modifiers": ["fast_sortable"] },
      { "name": "feature", "column_type": "text", "modifiers": ["full_text"] },
      { "name": "tech1", "column_type": "text", "modifiers": ["full_text", "nullable"] },
      { "name": "tech2", "column_type": "text", "modifiers": ["full_text", "nullable"] },
      { "name": "also_buy", "column_type": "text", "modifiers": ["nullable"] },
      { "name": "also_view", "column_type": "text", "modifiers": ["nullable"] },
      { "name": "brand_string", "column_type": "text", "modifiers": ["full_text", "nullable"] },
      { "name": "brand", "column_type": "tree", "modifiers": ["equals", "fast_sortable"] },
      { "name": "rank_position", "column_type": "ulong", "modifiers": ["equals", "fast_sortable", "nullable"] },
      { "name": "rank_facet", "column_type": "tree", "modifiers": ["equals", "fast_sortable", "nullable"] },
      { "name": "category", "column_type": "tree", "modifiers": ["equals", "fast_sortable"] },
      { "name": "image_url", "column_type": "text", "modifiers": ["nullable"] },
      { "name": "image_url_high_res", "column_type": "text", "modifiers": ["nullable"] }
    ]
  }
}
EOF

# Send schema via HTTP POST
curl -v -X POST "$INDEXER_URL" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  --data-raw "$SCHEMA_JSON"
