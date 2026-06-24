#!/bin/bash
curl -s -H 'Content-Type: application/json' \
  -d '{"model": "models/text-embedding-004", "content": {"parts": [{"text": "Hello world"}]}}' \
  "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent?key=$GOOGLE_API_KEY"
