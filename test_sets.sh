#!/bin/bash

# Test script to verify sets functionality
set -e

echo "Testing Bert Sets Functionality"
echo "================================"
echo ""

# Check if sets directory exists
if [ -d "docs/bert/prompts/sets" ]; then
    echo "✓ Sets directory exists"
    echo "  Found sets:"
    ls -1 docs/bert/prompts/sets/*.yaml 2>/dev/null | sed 's/.*\//  - /' || echo "  (none)"
else
    echo "✗ Sets directory not found"
    exit 1
fi

echo ""

# Check if library directory exists
if [ -d "docs/bert/prompts/library" ]; then
    echo "✓ Library directory exists"
    FILE_COUNT=$(find docs/bert/prompts/library -type f -name "*.md" | wc -l)
    echo "  Files: $FILE_COUNT markdown files"
else
    echo "✗ Library directory not found"
    exit 1
fi

echo ""

# Check if binary is built
if [ -f "target/debug/bert" ]; then
    echo "✓ Binary exists"
    echo "  Version info:"
    ./target/debug/bert --version | sed 's/^/  /'
else
    echo "✗ Binary not found (run 'cargo build' first)"
    exit 1
fi

echo ""

# Test reading a set file
if [ -f "docs/bert/prompts/sets/foo.yaml" ]; then
    echo "✓ Test set 'foo' found"
    echo "  Contents:"
    cat docs/bert/prompts/sets/foo.yaml | sed 's/^/  /'
else
    echo "✗ Test set not found"
fi

echo ""
echo "================================"
echo "All basic checks passed!"
echo ""
echo "To test the TUI manually:"
echo "  ./target/debug/bert tui"
echo ""
echo "Keys to test:"
echo "  \` (backtick) - Toggle Library ↔ Sets"
echo "  Space/Enter  - Add file or load set"
echo "  s (in Queue) - Save buffer as set"
echo "  ?            - Show help"
echo "  q            - Quit"
