#!/usr/bin/env bash

# Load environment variables from .env
export $(grep -v '^#' .env | xargs)

SEARCHER_URL="http://localhost:$SEARCHER_HTTP_PORT"

curl -X POST "$SEARCHER_URL/v1/select" \
  -H "Content-Type: application/json" \
  -d '{
    "select": ["title", "timestamp_creation_ms"],
    "filter": "*",
    "functions": ["timestamp_creation_ms+10"],
    "from": "electronics",
    "sort": "-ln(price + 1) + 10 * exp(-0.01 * ((now_ms() - 202176000000 - timestamp_creation_ms) / 86400000))",
    "limit": 20
    }' | jq
