#!/bin/bash
curl -s -H 'Content-Type: application/json' -d '{"contents":[{"parts":[{"text":"hello"}]}]}' "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key=$KEY"
