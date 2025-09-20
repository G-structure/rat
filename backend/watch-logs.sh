#!/bin/bash

# Watch RAT logs in real-time
# Usage: ./watch-logs.sh [grep-pattern]

LOG_FILE="logs/rat.log"

# Create logs directory if it doesn't exist
mkdir -p logs

# If a grep pattern is provided, filter the logs
if [ $# -eq 0 ]; then
    echo "Watching all logs in $LOG_FILE..."
    echo "Press Ctrl+C to exit"
    echo "----------------------------------------"
    tail -f "$LOG_FILE"
else
    echo "Watching logs in $LOG_FILE filtered by: $1"
    echo "Press Ctrl+C to exit"
    echo "----------------------------------------"
    tail -f "$LOG_FILE" | grep --line-buffered "$1"
fi
