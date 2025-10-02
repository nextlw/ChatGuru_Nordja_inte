# Quick Test Guide

## Run Tests

```bash
# Show available scenarios
node test-scenarios.js

# Run specific scenario
node test-scenarios.js 1

# Run all scenarios
node test-scenarios.js all

# AI-specific tests
node test-ai-features.js 1
node test-ai-features.js all
```

## Monitor Logs

```bash
# Real-time logs
gcloud run logs tail chatguru-clickup-middleware --region=southamerica-east1

# Filter AI operations
gcloud run logs read chatguru-clickup-middleware --region=southamerica-east1 | grep "🎤\|🧠\|📤"
```

## Check Results

1. **Logs**: Look for success patterns (🎤, 🧠, 📤) and error patterns (❌, ⚠️)
2. **ClickUp**: Verify tasks created in list 901300373349
3. **Response**: All JSON responses with proper `action`, `task_id`, `is_activity`

See [TESTING.md](TESTING.md) for complete documentation.
