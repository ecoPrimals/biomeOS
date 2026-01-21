#!/bin/bash
# Squirrel Wrapper Script - Proper Environment Variable Passing
# Date: January 21, 2026

# Set environment variables
export ANTHROPIC_API_KEY="sk-ant-REDACTED"
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
unset AI_PROVIDER_SOCKETS  # Critical: Don't treat Songbird as AI provider

# Use exec to replace this shell with Squirrel
# This ensures environment variables are properly inherited
exec /home/eastgate/Development/ecoPrimals/plasmidBin/primals/squirrel/squirrel-x86_64 server --socket /tmp/squirrel-nat0.sock "$@"

