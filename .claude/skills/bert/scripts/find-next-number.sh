#!/bin/bash
# find-next-number.sh - Universal numbering for specs and tasks
# Usage: bash find-next-number.sh <config_file>
# Returns: Next available number (2-digit padded) by scanning BOTH specs and tasks

set -e

CONFIG_FILE="$1"

if [[ -z "$CONFIG_FILE" ]]; then
    echo '{"error": "Usage: find-next-number.sh <config_file>"}' >&2
    exit 1
fi

# Read all relevant directories from config
TASKS_DIR=$(grep "tasks_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
ARCHIVE_TASKS=$(grep "archive_tasks_directory:" "$CONFIG_FILE" | awk '{print $2}')
SPECS_DIR=$(grep "specs_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
ARCHIVE_SPECS=$(grep "archive_specs_directory:" "$CONFIG_FILE" | awk '{print $2}')

MAX_NUM=0

# Scan active tasks (task-03-*.md -> 03)
if [[ -d "$TASKS_DIR" ]]; then
    while IFS= read -r -d '' file; do
        filename=$(basename "$file")
        # Extract top-level task number only (task-03-*.md -> 03, ignore task-03.1-*.md)
        if [[ "$filename" =~ ^task-([0-9]+)-.*\.md$ ]]; then
            num="${BASH_REMATCH[1]}"
            num=$((10#$num))  # Remove leading zeros
            if [[ $num -gt $MAX_NUM ]]; then
                MAX_NUM=$num
            fi
        fi
    done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-*.md" -print0 2>/dev/null)
fi

# Scan archived tasks
if [[ -d "$ARCHIVE_TASKS" ]]; then
    while IFS= read -r -d '' file; do
        filename=$(basename "$file")
        if [[ "$filename" =~ ^task-([0-9]+)-.*\.md$ ]]; then
            num="${BASH_REMATCH[1]}"
            num=$((10#$num))
            if [[ $num -gt $MAX_NUM ]]; then
                MAX_NUM=$num
            fi
        fi
    done < <(find "$ARCHIVE_TASKS" -maxdepth 1 -name "task-*.md" -print0 2>/dev/null)
fi

# Scan active specs (spec-12/ -> 12)
if [[ -d "$SPECS_DIR" ]]; then
    while IFS= read -r -d '' dir; do
        dirname=$(basename "$dir")
        # Extract spec number (spec-12 -> 12)
        if [[ "$dirname" =~ ^spec-([0-9]+)$ ]]; then
            num="${BASH_REMATCH[1]}"
            num=$((10#$num))
            if [[ $num -gt $MAX_NUM ]]; then
                MAX_NUM=$num
            fi
        fi
    done < <(find "$SPECS_DIR" -maxdepth 1 -type d -name "spec-*" -print0 2>/dev/null)
fi

# Scan archived specs
if [[ -d "$ARCHIVE_SPECS" ]]; then
    while IFS= read -r -d '' dir; do
        dirname=$(basename "$dir")
        if [[ "$dirname" =~ ^spec-([0-9]+)$ ]]; then
            num="${BASH_REMATCH[1]}"
            num=$((10#$num))
            if [[ $num -gt $MAX_NUM ]]; then
                MAX_NUM=$num
            fi
        fi
    done < <(find "$ARCHIVE_SPECS" -maxdepth 1 -type d -name "spec-*" -print0 2>/dev/null)
fi

# Next number is max + 1, padded to 2 digits
NEXT_NUM=$((MAX_NUM + 1))
printf "%02d" "$NEXT_NUM"
