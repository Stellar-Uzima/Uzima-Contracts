#!/bin/bash

# Function to log messages in a structured format (JSON)
# Usage: log <level> <message>
# Example: log "INFO" "Deploying contract..."
log() {
    local level="$1"
    local message="$2"
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    # Create a JSON object
    echo "{\"timestamp\": \"$timestamp\", \"level\": \"$level\", \"message\": \"$message\"}"
}