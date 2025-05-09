#!/usr/bin/env bash

# Load environment variables from .env
export $(grep -v '^#' .env | xargs)

SEARCHER_URL="http://localhost:$SEARCHER_HTTP_PORT"

curl -X POST "$SEARCHER_URL/v1/select" \
  -H "Content-Type: application/json" \
  -d '{
    "select": ["title", "timestamp_creation_ms"],
    "filter": "*",
    "functions": ["sqrt(timestamp_creation_ms)"],
    "from": "electronics",
    "sort": "sqrt(timestamp_creation_ms)",
    "limit": 20
    }' | jq
