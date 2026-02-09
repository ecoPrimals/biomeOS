#!/bin/bash
# NUCLEUS Multi-AI Provider Validation
# Tests: Anthropic, OpenAI, HuggingFace, and Local compute
#
# Usage: ./scripts/validate_multi_ai.sh

set -e
cd "$(dirname "$0")/.."

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║              NUCLEUS MULTI-AI PROVIDER VALIDATION                ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# API Keys
HF_TOKEN="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"

# Sockets
SQUIRREL_SOCKET="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos/squirrel-1894e909e454.sock"
SONGBIRD_SOCKET="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos/songbird-node-alpha.sock"
TOADSTOOL_SOCKET="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos/toadstool-1894e909e454.jsonrpc.sock"

PASSED=0
FAILED=0

test_ai() {
    local name="$1"
    local socket="$2"
    local request="$3"
    local expect="$4"
    
    printf "  %-40s " "$name..."
    
    START=$(date +%s%N)
    RESULT=$(echo "$request" | timeout 30 nc -U "$socket" 2>/dev/null | head -1)
    END=$(date +%s%N)
    LATENCY=$(( (END - START) / 1000000 ))
    
    if echo "$RESULT" | grep -q "$expect"; then
        echo "✅ (${LATENCY}ms)"
        PASSED=$((PASSED + 1))
        # Show snippet of response
        echo "      → $(echo "$RESULT" | grep -o '"response":"[^"]*"' | head -c 80 || echo "$RESULT" | head -c 80)..."
    else
        echo "❌ (${LATENCY}ms)"
        FAILED=$((FAILED + 1))
        echo "      → $RESULT" | head -c 150
        echo ""
    fi
}

echo "═══════════════════════════════════════════════════════════════════"
echo "TEST 1: Anthropic Claude (Text Generation)"
echo "═══════════════════════════════════════════════════════════════════"
test_ai "Claude 3 Haiku - Simple math" "$SQUIRREL_SOCKET" \
    '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"What is 15+27? Answer with just the number.","model":"claude-3-haiku-20240307"},"id":1}' \
    "result"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "TEST 2: OpenAI GPT (Text Generation)"
echo "═══════════════════════════════════════════════════════════════════"

# OpenAI text via direct HTTP
echo "  Testing OpenAI GPT-4o-mini..."
OPENAI_BODY=$(echo -n '{"model":"gpt-4o-mini","messages":[{"role":"user","content":"What is 10*5? Answer with just the number."}],"max_tokens":10}' | base64 -w0)
OPENAI_KEY="${OPENAI_API_KEY:-sk-proj-REDACTED-aB7kj0-RxH1ZM20VGh5hck21Cz2zktasTGbGy7OxT3BlbkFJc7JB1rzlYBu-Mgp-ukYLz8nevjlQmetKQiEmgjfJTCXuhPkS8xA-LbrKPIFQSg5sNPlS5tsSoA}"
test_ai "GPT-4o-mini - Simple math" "$SONGBIRD_SOCKET" \
    "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api.openai.com/v1/chat/completions\",\"headers\":{\"Authorization\":\"Bearer $OPENAI_KEY\",\"Content-Type\":\"application/json\"},\"body\":\"$OPENAI_BODY\"},\"id\":1}" \
    "body"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "TEST 3: HuggingFace Inference API"
echo "═══════════════════════════════════════════════════════════════════"

# Sentiment Analysis
echo "  Testing HuggingFace distilbert..."
HF_BODY=$(echo -n '{"inputs":"biomeOS distributed AI is amazing!"}' | base64 -w0)
test_ai "DistilBERT Sentiment" "$SONGBIRD_SOCKET" \
    "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api-inference.huggingface.co/models/distilbert-base-uncased-finetuned-sst-2-english\",\"headers\":{\"Authorization\":\"Bearer $HF_TOKEN\",\"Content-Type\":\"application/json\"},\"body\":\"$HF_BODY\"},\"id\":1}" \
    "body"

# Text Generation (smaller model)
echo "  Testing HuggingFace GPT-2..."
HF_GPT2_BODY=$(echo -n '{"inputs":"The biomeOS ecosystem"}' | base64 -w0)
test_ai "GPT-2 Text Gen" "$SONGBIRD_SOCKET" \
    "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api-inference.huggingface.co/models/gpt2\",\"headers\":{\"Authorization\":\"Bearer $HF_TOKEN\",\"Content-Type\":\"application/json\"},\"body\":\"$HF_GPT2_BODY\"},\"id\":1}" \
    "body"

# Fill-Mask (BERT)
echo "  Testing HuggingFace BERT fill-mask..."
HF_BERT_BODY=$(echo -n '{"inputs":"The NUCLEUS system is [MASK]."}' | base64 -w0)
test_ai "BERT Fill-Mask" "$SONGBIRD_SOCKET" \
    "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api-inference.huggingface.co/models/bert-base-uncased\",\"headers\":{\"Authorization\":\"Bearer $HF_TOKEN\",\"Content-Type\":\"application/json\"},\"body\":\"$HF_BERT_BODY\"},\"id\":1}" \
    "body"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "TEST 4: Toadstool Local Compute"
echo "═══════════════════════════════════════════════════════════════════"
test_ai "Toadstool GPU Capabilities" "$TOADSTOOL_SOCKET" \
    '{"jsonrpc":"2.0","method":"toadstool.query_capabilities","params":{},"id":1}' \
    "compute"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "TEST 5: Coordinated Multi-AI Task"
echo "═══════════════════════════════════════════════════════════════════"
echo "  Testing sequential AI coordination..."

# Step 1: HuggingFace sentiment
printf "  Step 1: Analyze sentiment (HuggingFace)... "
HF_STEP=$(echo -n '{"inputs":"The distributed AI coordination is working perfectly!"}' | base64 -w0)
S1=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"http.post\",\"params\":{\"url\":\"https://api-inference.huggingface.co/models/distilbert-base-uncased-finetuned-sst-2-english\",\"headers\":{\"Authorization\":\"Bearer $HF_TOKEN\",\"Content-Type\":\"application/json\"},\"body\":\"$HF_STEP\"},\"id\":1}" | timeout 15 nc -U "$SONGBIRD_SOCKET" 2>/dev/null | head -1)
if echo "$S1" | grep -q "body"; then
    echo "✅"
    PASSED=$((PASSED + 1))
else
    echo "❌"
    FAILED=$((FAILED + 1))
fi

# Step 2: Claude summarize
printf "  Step 2: Summarize results (Anthropic)... "
S2=$(echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Briefly describe what a POSITIVE sentiment means in 10 words.","model":"claude-3-haiku-20240307","max_tokens":30},"id":1}' | timeout 15 nc -U "$SQUIRREL_SOCKET" 2>/dev/null | head -1)
if echo "$S2" | grep -q "result"; then
    echo "✅"
    PASSED=$((PASSED + 1))
else
    echo "❌"
    FAILED=$((FAILED + 1))
fi

# Step 3: Toadstool report compute
printf "  Step 3: Report compute status (Toadstool)... "
S3=$(echo '{"jsonrpc":"2.0","method":"toadstool.health","params":{},"id":1}' | timeout 5 nc -U "$TOADSTOOL_SOCKET" 2>/dev/null | head -1)
if echo "$S3" | grep -q "healthy"; then
    echo "✅"
    PASSED=$((PASSED + 1))
else
    echo "❌"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                      VALIDATION SUMMARY                          ║"
echo "╠═══════════════════════════════════════════════════════════════════╣"
printf "║  Total Tests: %-3d                                                ║\n" $((PASSED + FAILED))
printf "║  Passed:      %-3d ✅                                             ║\n" $PASSED
printf "║  Failed:      %-3d ❌                                             ║\n" $FAILED
echo "╠═══════════════════════════════════════════════════════════════════╣"
echo "║  AI PROVIDERS TESTED:                                            ║"
echo "║    • Anthropic Claude (text.generation)                          ║"
echo "║    • OpenAI GPT (text.generation)                                ║"
echo "║    • HuggingFace (sentiment, text-gen, fill-mask)               ║"
echo "║    • Toadstool (local compute coordination)                      ║"
echo "╠═══════════════════════════════════════════════════════════════════╣"
if [ $FAILED -eq 0 ]; then
    echo "║  🎉 MULTI-AI COORDINATION VALIDATED                              ║"
else
    echo "║  ⚠️  SOME PROVIDERS NEED ATTENTION                               ║"
fi
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Timestamp: $(date -Iseconds)"

exit $FAILED

