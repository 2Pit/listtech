#!/usr/bin/env bash

# Load environment variables from .env
export $(grep -v '^#' .env | xargs)

INDEXER_URL="http://localhost:$INDEXER_HTTP_PORT/v1/schema"

# Send schema via HTTP POST
curl -v -X GET "$INDEXER_URL/electronics" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json"
