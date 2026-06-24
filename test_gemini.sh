curl -s -H "Content-Type: application/json" -d '{
  "contents": [
    {"role": "user", "parts": [{"text": "hello"}]}
  ],
  "system_instruction": {
    "parts": [{"text": "You are a text tagger. Analyze the following user message and extract topical and structural tags. Format your response as a JSON object matching this schema:\n{\n  \"topical_tags\": [\"list\", \"of\", \"topics\"],\n  \"structural\": {\n    \"addressee\": \"Target of message (e.g. Cova, Bot, All, None)\",\n    \"intent\": \"Question, Statement, Command, etc.\"\n  }\n}"}]
  },
  "generationConfig": {
    "responseMimeType": "application/json"
  }
}' "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key=$KEY"
