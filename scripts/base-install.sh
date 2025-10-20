#!/usr/bin/env bash

# Bert installer - Task management for Claude Code
# Can be run locally or via: curl -fsSL https://raw.githubusercontent.com/ssr1ram/bert-sidecar/main/scripts/base-install.sh | bash

VERSION="3.0.0"
LAST_UPDATED="2025-10-20"

set -e  # Exit on error

# Determine if we're running from a local clone or via curl
SCRIPT_NAME="${BASH_SOURCE[0]}"

if [[ "$SCRIPT_NAME" == "bash" ]] || [[ "$SCRIPT_NAME" == "-bash" ]] || [[ "$SCRIPT_NAME" =~ ^/dev/fd/ ]] || [[ -z "$SCRIPT_NAME" ]]; then
    # Running via curl | bash
    INSTALL_MODE="remote"
    GITHUB_RAW_URL="https://raw.githubusercontent.com/ssr1ram/bert-sidecar/main"
    echo "Running in REMOTE mode (downloading from GitHub)"
else
    # Running from a file - check if it's a local clone
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)"
    SOURCE_REPO="$(dirname "$SCRIPT_DIR")"

    if [ -d "$SOURCE_REPO/.claude/commands/bert" ]; then
        INSTALL_MODE="local"
        echo "Running in LOCAL mode (using cloned repository)"
    else
        INSTALL_MODE="remote"
        GITHUB_RAW_URL="https://raw.githubusercontent.com/ssr1ram/bert-sidecar/main"
        echo "Running in REMOTE mode (downloading from GitHub)"
    fi
fi

echo "========================================="
echo "Bert - Task Management Installer"
echo "Version: $VERSION (Updated: $LAST_UPDATED)"
echo "========================================="
echo ""

# Step 1: Determine target directory
CURRENT_DIR=$(pwd)
echo "Current directory: $CURRENT_DIR"
echo ""
read -p "Is this where you want to install Bert? (Y/n): " USE_CURRENT < /dev/tty

USE_CURRENT=${USE_CURRENT:-Y}  # Default to Y if empty

if [[ "$USE_CURRENT" =~ ^[Yy]$ ]]; then
    TARGET_REPO="$CURRENT_DIR"
else
    read -p "Enter the path to your target repository: " TARGET_REPO < /dev/tty
    # Expand ~ to home directory
    TARGET_REPO="${TARGET_REPO/#\~/$HOME}"
fi

# Validate target directory
if [ ! -d "$TARGET_REPO" ]; then
    echo "Error: Directory '$TARGET_REPO' does not exist."
    exit 1
fi

echo ""
echo "Target repository: $TARGET_REPO"
echo ""

# Step 2: Create target directory structure
COMMANDS_DIR="$TARGET_REPO/.claude/commands/bert"
SKILLS_DIR="$TARGET_REPO/.claude/skills/bert"
AGENTS_DIR="$SKILLS_DIR/agents"

echo "Creating directory structure..."
mkdir -p "$COMMANDS_DIR"
mkdir -p "$AGENTS_DIR"

# Step 3: Copy or download files
echo "Installing Bert files..."

if [ "$INSTALL_MODE" = "local" ]; then
    echo "From: $SOURCE_REPO/.claude/"
    echo "To:   $TARGET_REPO/.claude/"
    echo ""

    # Copy commands
    cp -pr "$SOURCE_REPO/.claude/commands/bert"/* "$COMMANDS_DIR/"

    # Copy skills
    cp -pr "$SOURCE_REPO/.claude/skills/bert"/* "$SKILLS_DIR/"

    if [ $? -eq 0 ]; then
        echo "✓ Successfully copied all files"
    else
        echo "✗ Error: Failed to copy files"
        exit 1
    fi
else
    echo "From: GitHub (ssr1ram/bert-sidecar)"
    echo "To:   $TARGET_REPO/.claude/"
    echo ""

    # Download command files
    COMMAND_FILES=("plan.md" "spec.md" "task.md")
    CMD_BASE_URL="$GITHUB_RAW_URL/.claude/commands/bert"

    echo "Downloading commands..."
    for FILE in "${COMMAND_FILES[@]}"; do
        echo "  Downloading $FILE..."
        if curl -fsSL "$CMD_BASE_URL/$FILE" -o "$COMMANDS_DIR/$FILE"; then
            echo "  ✓ Downloaded $FILE"
        else
            echo "  ✗ Failed to download $FILE"
            exit 1
        fi
    done

    # Download skill files
    SKILL_FILES=("skill.md" "skill.yml")
    SKILL_BASE_URL="$GITHUB_RAW_URL/.claude/skills/bert"

    echo ""
    echo "Downloading skill files..."
    for FILE in "${SKILL_FILES[@]}"; do
        echo "  Downloading $FILE..."
        if curl -fsSL "$SKILL_BASE_URL/$FILE" -o "$SKILLS_DIR/$FILE"; then
            echo "  ✓ Downloaded $FILE"
        else
            echo "  ✗ Failed to download $FILE"
            exit 1
        fi
    done

    # Download agent files
    AGENT_FILES=("requirements-gatherer.md" "spec-iterator.md" "task-proposer.md" "task-decomposer.md")
    AGENT_BASE_URL="$GITHUB_RAW_URL/.claude/skills/bert/agents"

    echo ""
    echo "Downloading agent files..."
    for FILE in "${AGENT_FILES[@]}"; do
        echo "  Downloading $FILE..."
        if curl -fsSL "$AGENT_BASE_URL/$FILE" -o "$AGENTS_DIR/$FILE"; then
            echo "  ✓ Downloaded $FILE"
        else
            echo "  ✗ Failed to download $FILE"
            exit 1
        fi
    done

    echo ""
    echo "✓ Successfully downloaded all files"
fi

# Verify installation
echo ""
echo "========================================="
echo "Installation complete!"
echo "========================================="
echo ""
echo "Installed files:"
echo ""
echo "Commands (.claude/commands/bert/):"
ls -la "$COMMANDS_DIR" | tail -n +4 | awk '{print "  - " $9}'
echo ""
echo "Skills (.claude/skills/bert/):"
ls -la "$SKILLS_DIR" | grep -v "^d" | tail -n +2 | awk '{print "  - " $9}'
echo ""
echo "Agents (.claude/skills/bert/agents/):"
ls -la "$AGENTS_DIR" | tail -n +4 | awk '{print "  - " $9}'

echo ""
echo "========================================="
echo "Next steps:"
echo "========================================="
echo ""
echo "1. Restart Claude Code"
echo ""
echo "2. Available commands:"
echo "   - /bert:task create \"description\"     # Create task"
echo "   - /bert:task execute 12                # Work on task"
echo "   - /bert:spec new \"description\"        # Start spec"
echo "   - /bert:spec iterate 12                # Refine spec"
echo "   - /bert:plan                           # Setup product context"
echo ""
echo "3. Documentation:"
echo "   See README.md for full workflow examples"
echo ""
