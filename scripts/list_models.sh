#!/bin/bash
API_KEY=$(kubectl get secret starbunk-secrets -n starbunk -o jsonpath='{.data.GOOGLE_API_KEY}' | base64 -d)
curl -s "https://generativelanguage.googleapis.com/v1beta/models?key=$API_KEY" | jq '.models[] | select(.name | contains("embed")) | .name'
