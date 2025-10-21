# Agent-OS Integration: Technical Implementation Details

**Date**: 2025-10-21
**Companion to**: `agent-os-integration-recommendations.md`

This document provides specific code changes, technical specifications, and implementation details for the recommendations.

---

## Table of Contents

1. [Implementation Roadmap](#implementation-roadmap)
2. [Path Configuration Implementation](#path-configuration-implementation)
3. [Workflow Updates](#workflow-updates)
4. [Command Path Overrides](#command-path-overrides)
5. [Installation Script Changes](#installation-script-changes)
6. [Workflows API Specification](#workflows-api-specification)
7. [Example Integrations](#example-integrations)

---

## Implementation Roadmap

### Phase 1: Path Configuration (v2.2.0)

**Goal**: Make paths configurable without breaking existing users

**Timeline**: 2-3 weeks
**Breaking Changes**: None

**Tasks**:
1. Add `paths` section to `config.yml`
2. Add helper functions to `common-functions.sh`
3. Update workflows to read from environment variables
4. Add fallbacks to hardcoded defaults (backward compatible)
5. Update installation script to export paths as env vars
6. Document new path configuration

**Deliverables**:
- Updated config.yml with paths section
- Modified workflows using env vars
- Documentation in README and installation guide

---

### Phase 2: Workflows API Documentation (v2.2.0 or v2.3.0)

**Goal**: Document workflows as public APIs

**Timeline**: 1 week
**Breaking Changes**: None

**Tasks**:
1. Create `docs/workflows-api.md`
2. Document each public workflow (inputs, outputs, examples)
3. Add version markers to workflows
4. Add "Integration Guide" with examples
5. Document stability guarantees

**Deliverables**:
- Complete workflows API documentation
- Integration guide with examples
- Version markers in workflow files

---

### Phase 3: Command Path Overrides (v2.3.0)

**Goal**: Allow per-command path customization

**Timeline**: 1-2 weeks
**Breaking Changes**: None

**Tasks**:
1. Add argument parsing for `--specs-dir`, `--product-dir` flags
2. Update all commands to accept overrides
3. Pass overrides to workflows as env vars
4. Add validation for user-provided paths
5. Document flag usage in help text

**Deliverables**:
- Updated commands with flag support
- Help documentation for flags
- Validation and security checks

---

### Phase 4: Modular Usage Guide (v2.3.0)

**Goal**: Document how to use parts of Agent-OS

**Timeline**: 3-5 days
**Breaking Changes**: None

**Tasks**:
1. Create `docs/modular-usage.md`
2. Document use cases (specs only, implementation only, etc.)
3. Add integration examples (BERT, custom tools)
4. Update README with "You don't need to use everything"

**Deliverables**:
- Modular usage documentation
- Integration examples
- Updated README

---


**Key Point**: All phases are backward compatible. Existing users see zero breaking changes.

---

## Path Configuration Implementation

### 1. Update config.yml

**Add paths section**:

```yaml
# config.yml
version: 2.2.0
base_install: true

# NEW: Path configuration
paths:
  # Specs
  specs_directory: "agent-os/specs"
  specs_naming: "{date}-{name}"        # Template: YYYY-MM-DD-spec-name

  # Product
  product_directory: "agent-os/product"

  # Task files
  tasks_file: "tasks.md"               # Within spec directory

  # Implementation tracking
  implementation_directory: "implementation"

# Existing config...
claude_code_commands: true
use_claude_code_subagents: true
standards_as_claude_code_skills: true
profile: default
```

### 2. Add Helper Functions to common-functions.sh

```bash
# -----------------------------------------------------------------------------
# Path Configuration Functions
# -----------------------------------------------------------------------------

# Get path configuration with fallback
get_path_config() {
    local config_key=$1
    local default_value=$2
    local config_file="${BASE_DIR}/config.yml"

    get_yaml_value "$config_file" "paths.${config_key}" "$default_value"
}

# Build spec path from template
build_spec_path() {
    local spec_name=$1
    local date=$2

    local specs_dir=$(get_path_config "specs_directory" "agent-os/specs")
    local naming_template=$(get_path_config "specs_naming" "{date}-{name}")

    # Replace template variables
    local dated_name=$(echo "$naming_template" | sed "s/{date}/$date/" | sed "s/{name}/$spec_name/")

    echo "$specs_dir/$dated_name"
}

# Export path configuration as environment variables
export_path_config() {
    export SPECS_DIRECTORY=$(get_path_config "specs_directory" "agent-os/specs")
    export PRODUCT_DIRECTORY=$(get_path_config "product_directory" "agent-os/product")
    export SPEC_NAMING_TEMPLATE=$(get_path_config "specs_naming" "{date}-{name}")
    export TASKS_FILE=$(get_path_config "tasks_file" "tasks.md")
    export IMPLEMENTATION_DIR=$(get_path_config "implementation_directory" "implementation")

    if [[ "$VERBOSE" == "true" ]]; then
        print_verbose "Exported path configuration:"
        print_verbose "  SPECS_DIRECTORY=$SPECS_DIRECTORY"
        print_verbose "  PRODUCT_DIRECTORY=$PRODUCT_DIRECTORY"
        print_verbose "  SPEC_NAMING_TEMPLATE=$SPEC_NAMING_TEMPLATE"
    fi
}
```

---

## Workflow Updates

### 1. Update workflows/specification/initialize-spec.md

**Before**:
```bash
SPEC_PATH="agent-os/specs/$DATED_SPEC_NAME"
mkdir -p $SPEC_PATH/planning
mkdir -p $SPEC_PATH/planning/visuals
```

**After**:
```bash
# Read from environment or use defaults
SPECS_DIR="${SPECS_DIRECTORY:-agent-os/specs}"
SPEC_NAMING="${SPEC_NAMING_TEMPLATE:-{date}-{name}}"

# Get today's date
TODAY=$(date +%Y-%m-%d)

# Determine spec name from description
SPEC_NAME="[kebab-case-name]"

# Build dated name from template
DATED_SPEC_NAME=$(echo "$SPEC_NAMING" | sed "s/{date}/$TODAY/" | sed "s/{name}/$SPEC_NAME/")

# Build full path
SPEC_PATH="$SPECS_DIR/$DATED_SPEC_NAME"

# Create folder structure
mkdir -p "$SPEC_PATH/planning"
mkdir -p "$SPEC_PATH/planning/visuals"

# Store path for output
echo "Created spec folder: $SPEC_PATH"
```

**Output message update**:
```markdown
✅ I have initialized the spec folder at `${SPEC_PATH}`.

NEXT STEP 👉 Run the command, 2-research-spec.md
```

Now uses actual path instead of hardcoded "agent-os/specs/"!

---

### 2. Update workflows/planning/create-product-mission.md

**Before**:
```bash
cat > agent-os/product/mission.md << EOF
...
EOF
```

**After**:
```bash
# Read from environment or use default
PRODUCT_DIR="${PRODUCT_DIRECTORY:-agent-os/product}"

# Ensure directory exists
mkdir -p "$PRODUCT_DIR"

# Create file
cat > "$PRODUCT_DIR/mission.md" << EOF
...
EOF
```

**Apply same pattern to**:
- `create-product-roadmap.md`
- `create-product-tech-stack.md`

---

### 3. Update workflows/specification/write-spec.md

**Before**:
```bash
cat agent-os/specs/[current-spec]/planning/requirements.md
ls -la agent-os/specs/[current-spec]/planning/visuals/
```

**After**:
```bash
# Expect SPEC_PATH to be set by caller
if [[ -z "$SPEC_PATH" ]]; then
    echo "Error: SPEC_PATH not set"
    exit 1
fi

# Read requirements
cat "$SPEC_PATH/planning/requirements.md"

# Check for visuals
ls -la "$SPEC_PATH/planning/visuals/" 2>/dev/null
```

**Write spec**:
```bash
# Before
cat > agent-os/specs/[current-spec]/spec.md << EOF

# After
cat > "$SPEC_PATH/spec.md" << EOF
```

---

### 4. Update workflows/implementation/implement-tasks.md

**Before**:
```markdown
Update `agent-os/specs/[this-spec]/tasks.md` to mark tasks as done
```

**After**:
```markdown
# Expect SPEC_PATH and TASKS_FILE to be set
TASKS_FILE="${TASKS_FILE:-tasks.md}"

# Update tasks
Update `${SPEC_PATH}/${TASKS_FILE}` to mark tasks as done
```

---

## Command Path Overrides

### Update Commands to Accept Flags

**Example: .claude/commands/agent-os/shape-spec.md**

**Add argument parsing**:

```markdown
You are helping to shape a specification.

## Parse Arguments

```bash
# Initialize with defaults from environment or hardcoded
SPECS_DIR="${SPECS_DIRECTORY:-agent-os/specs}"
PRODUCT_DIR="${PRODUCT_DIRECTORY:-agent-os/product}"
SPEC_NAMING="${SPEC_NAMING_TEMPLATE:-{date}-{name}}"
DESCRIPTION=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --specs-dir)
      SPECS_DIR="$2"
      shift 2
      ;;
    --product-dir)
      PRODUCT_DIR="$2"
      shift 2
      ;;
    --naming)
      SPEC_NAMING="$2"
      shift 2
      ;;
    --help)
      echo "Usage: /shape-spec [OPTIONS] <description>"
      echo ""
      echo "Options:"
      echo "  --specs-dir PATH      Override specs directory (default: from config)"
      echo "  --product-dir PATH    Override product directory (default: from config)"
      echo "  --naming TEMPLATE     Override naming template (default: {date}-{name})"
      echo ""
      echo "Example:"
      echo "  /shape-spec 'authentication system'"
      echo "  /shape-spec --specs-dir docs/myapp/specs 'authentication'"
      exit 0
      ;;
    *)
      DESCRIPTION="$1"
      shift
      ;;
  esac
done

# Export for workflows
export SPECS_DIRECTORY="$SPECS_DIR"
export PRODUCT_DIRECTORY="$PRODUCT_DIR"
export SPEC_NAMING_TEMPLATE="$SPEC_NAMING"
```

## Execute Phases

{{PHASE 1: @agent-os/commands/shape-spec/1-initialize-spec.md}}

{{PHASE 2: @agent-os/commands/shape-spec/2-shape-spec.md}}
```

**Apply similar pattern to**:
- `/plan-product`
- `/write-spec`
- `/create-tasks`
- `/implement-tasks`

---

## Installation Script Changes

### Update project-install.sh

**After reading config, export paths**:

```bash
# Read configuration
read_project_config

# NEW: Export path configuration
export_path_config

# Compile commands with path variables available
install_claude_code_commands
```

**In compile_command function**:

```bash
compile_command() {
    local source=$1
    local dest=$2
    local base_dir=$3
    local profile=$4

    # Inject path configuration at top of compiled command
    {
        echo "# Agent-OS Path Configuration (from config.yml)"
        echo "# Override these with --specs-dir, --product-dir flags"
        echo "SPECS_DIRECTORY=\"${SPECS_DIRECTORY}\""
        echo "PRODUCT_DIRECTORY=\"${PRODUCT_DIRECTORY}\""
        echo "SPEC_NAMING_TEMPLATE=\"${SPEC_NAMING_TEMPLATE}\""
        echo "TASKS_FILE=\"${TASKS_FILE}\""
        echo ""

        # Then include the command content with workflows embedded
        compile_with_workflows "$source" "$base_dir" "$profile"
    } > "$dest"
}
```

This ensures every command has path configuration available by default.

---

## Workflows API Specification

### Create docs/workflows-api.md

```markdown
# Agent-OS Workflows API v1.0

Public workflows designed for external tool integration.

## Stability Guarantees

Workflows marked as **Stable** will:
- Maintain backward compatibility within major version
- Accept same input format
- Produce same output format
- Breaking changes will increment major version (v2.0, v3.0, etc.)

## Specification Workflows

### workflows/specification/initialize-spec

**Version**: 1.0
**Stability**: Stable

**Purpose**: Create a new spec folder structure

**Inputs** (environment variables):
```bash
SPECS_DIRECTORY      # Where to create spec (default: agent-os/specs)
SPEC_NAME            # Kebab-case spec name (required)
SPEC_NAMING_TEMPLATE # Naming pattern (default: {date}-{name})
```

**Outputs**:
- Creates directory: `$SPECS_DIRECTORY/$DATED_SPEC_NAME/`
- Creates subdirectories: `planning/`, `planning/visuals/`
- Returns: Full path to created spec (via echo)

**Example**:
```bash
export SPECS_DIRECTORY="docs/myapp/specs"
export SPEC_NAME="user-authentication"
{{workflows/specification/initialize-spec}}
```

**Returns**: `docs/myapp/specs/2025-10-21-user-authentication`

---

### workflows/specification/write-spec

**Version**: 1.0
**Stability**: Stable

**Purpose**: Generate spec.md from requirements.md

**Inputs**:
```bash
SPEC_PATH            # Path to spec directory (required)
```

**Reads**:
- `$SPEC_PATH/planning/requirements.md`
- `$SPEC_PATH/planning/visuals/*` (if present)

**Outputs**:
- Creates: `$SPEC_PATH/spec.md`

**Example**:
```bash
export SPEC_PATH="docs/myapp/specs/2025-10-21-user-auth"
{{workflows/specification/write-spec}}
```

---

## Planning Workflows

### workflows/planning/create-product-mission

**Version**: 1.0
**Stability**: Stable

**Purpose**: Create product mission document

**Inputs**:
```bash
PRODUCT_DIRECTORY    # Where to create file (default: agent-os/product)
```

**Outputs**:
- Creates: `$PRODUCT_DIRECTORY/mission.md`

**Example**:
```bash
export PRODUCT_DIRECTORY="docs/myapp/product"
{{workflows/planning/create-product-mission}}
```

---

## Implementation Workflows

### workflows/implementation/implement-tasks

**Version**: 1.0
**Stability**: Stable

**Purpose**: Implement tasks from task list

**Inputs**:
```bash
SPEC_PATH            # Path to spec directory (required)
TASK_NUMBERS         # Which tasks to implement (optional, default: all)
TASKS_FILE           # Task file name (default: tasks.md)
SKIP_VERIFICATION    # Skip verification phase (default: false)
```

**Reads**:
- `$SPEC_PATH/$TASKS_FILE`
- `$SPEC_PATH/spec.md`
- `$SPEC_PATH/planning/requirements.md`

**Outputs**:
- Implements code
- Updates task checkboxes in `$TASKS_FILE`
- Returns: List of files changed (via echo)

**Example**:
```bash
export SPEC_PATH="docs/myapp/specs/2025-10-21-auth"
export TASK_NUMBERS="1,2,3"
export SKIP_VERIFICATION="true"
{{workflows/implementation/implement-tasks}}
```

---

## Calling Workflows from External Tools

### From Shell Scripts

```bash
#!/bin/bash
source /path/to/agent-os/workflows/specification/initialize-spec.md
```

### From Claude Code Commands

```markdown
# Call workflow with custom paths
export SPECS_DIRECTORY="docs/bert/specs"
export SPEC_NAME="authentication"

{{workflows/specification/initialize-spec}}
```

### From Other AI Coding Tools

Include workflow content in prompts:

```markdown
Execute the following workflow with these parameters:

SPECS_DIRECTORY: docs/myapp/specs
SPEC_NAME: user-profile

[paste content of initialize-spec.md]
```
```

---

## Example Integrations

### Example 1: BERT Integration

**BERT's /bert:spec new command**:

```markdown
# .claude/commands/bert/spec.md

## Subcommand: new

Parse description from arguments.

Call Agent-OS spec initialization:

```bash
# Set BERT's paths
export SPECS_DIRECTORY="docs/bert/specs"
export PRODUCT_DIRECTORY="docs/bert/product"
export SPEC_NAME="[kebab-case from description]"

# Call Agent-OS workflow
{{workflows/specification/initialize-spec}}

# Get returned path
SPEC_PATH=$(# last output from workflow)

# Output BERT-style message
echo "✓ Created spec at $SPEC_PATH"
echo "Fill in requirements.md, then: /bert:spec iterate"
```
```

**Result**: BERT uses `docs/bert/specs/` while Agent-OS uses `agent-os/specs/`

---

### Example 2: Custom Spec-Only Tool

**Tool that only uses Agent-OS for spec writing**:

```markdown
# my-tool/commands/spec-write.md

The user wants to create a spec from requirements.

1. Check if requirements.md exists in current directory
2. If yes, call Agent-OS spec writer:

```bash
export SPEC_PATH="$(pwd)"
{{workflows/specification/write-spec}}
```

3. Output: "Spec created at ./spec.md"
```

**Result**: Tool uses Agent-OS spec writing without full Agent-OS installation

---

### Example 3: Enterprise with Custom Paths

**Enterprise team with specific structure**:

```yaml
# config.yml in their Agent-OS base installation
paths:
  specs_directory: "specifications"          # Not agent-os/specs
  specs_naming: "{name}-{date}"              # Reverse order
  product_directory: "product-docs"          # Custom name
```

**Result**: All commands use `specifications/` and `product-docs/` directories

---

## Testing Path Configuration

### Test Script

```bash
#!/bin/bash
# test-path-config.sh

# Test 1: Default paths
echo "Test 1: Default paths"
unset SPECS_DIRECTORY PRODUCT_DIRECTORY
export SPEC_NAME="test-spec"
source workflows/specification/initialize-spec.md
# Expected: agent-os/specs/2025-10-21-test-spec

# Test 2: Custom paths
echo "Test 2: Custom paths"
export SPECS_DIRECTORY="custom/specs"
export PRODUCT_DIRECTORY="custom/product"
export SPEC_NAME="test-spec"
source workflows/specification/initialize-spec.md
# Expected: custom/specs/2025-10-21-test-spec

# Test 3: Custom naming
echo "Test 3: Custom naming"
export SPECS_DIRECTORY="specs"
export SPEC_NAMING_TEMPLATE="{name}"  # No date
export SPEC_NAME="test-spec"
source workflows/specification/initialize-spec.md
# Expected: specs/test-spec

echo "All tests passed!"
```

---

## Migration Guide for Existing Projects

### Upgrading to v2.2.0 with Path Config

**For existing Agent-OS users**:

1. **Update base installation**:
   ```bash
   cd ~/agent-os
   git pull
   # Or re-run base installation
   ```

2. **Update project** (backward compatible):
   ```bash
   cd ~/my-project
   ~/agent-os/scripts/project-update.sh
   ```

3. **Optional: Customize paths**:
   ```bash
   # Edit ~/agent-os/config.yml
   vim ~/agent-os/config.yml

   # Add custom paths
   paths:
     specs_directory: "docs/specs"
     product_directory: "docs/product"
   ```

4. **Reinstall project to apply new paths**:
   ```bash
   ~/agent-os/scripts/project-install.sh --re-install
   ```

**Existing specs will NOT be moved** - commands will now use new paths for future specs.

To move existing specs:
```bash
mv agent-os/specs/* docs/specs/
mv agent-os/product/* docs/product/
```

---

## Backwards Compatibility Testing

### Test Matrix

| Scenario | Expected Result |
|----------|----------------|
| No config.yml paths section | Use hardcoded defaults (agent-os/specs) |
| Empty paths section | Use hardcoded defaults |
| Custom paths in config | Use custom paths |
| Path override flags | Override config paths |
| Mixed (some custom, some default) | Custom where specified, default otherwise |

### Test Cases

```bash
# Test 1: No configuration
rm config.yml
/shape-spec "test"
# Should create: agent-os/specs/2025-10-21-test

# Test 2: Partial configuration
echo "paths:
  specs_directory: custom/specs" > config.yml
/shape-spec "test"
# Should create: custom/specs/2025-10-21-test
# Should use: agent-os/product (default)

# Test 3: Full configuration
echo "paths:
  specs_directory: custom/specs
  product_directory: custom/product
  specs_naming: {name}-{date}" > config.yml
/shape-spec "test"
# Should create: custom/specs/test-2025-10-21

# Test 4: Command override
/shape-spec "test" --specs-dir override/specs
# Should create: override/specs/2025-10-21-test
```

---

## Performance Considerations

### Path Resolution Cost

**Negligible**: Reading env vars is instant
- `${SPECS_DIRECTORY:-default}` is a bash builtin operation
- No file I/O until path is actually used
- No performance impact vs hardcoded paths

### Configuration Loading

**Minimal**: Config read once at installation/compilation
- Path values embedded in compiled commands
- No runtime config reads needed
- Same performance as current implementation

---

## Security Considerations

### Path Traversal

**Validate user-provided paths**:

```bash
# In commands that accept --specs-dir
validate_path() {
    local path=$1

    # Reject absolute paths (optional - your choice)
    if [[ "$path" =~ ^\/ ]]; then
        echo "Error: Absolute paths not allowed"
        return 1
    fi

    # Reject path traversal attempts
    if [[ "$path" =~ \.\. ]]; then
        echo "Error: Path traversal not allowed"
        return 1
    fi

    return 0
}

if [[ -n "$USER_PROVIDED_PATH" ]]; then
    validate_path "$USER_PROVIDED_PATH" || exit 1
    SPECS_DIR="$USER_PROVIDED_PATH"
fi
```

### Environment Variable Injection

**Already safe**: Bash string handling prevents injection
- Variables are properly quoted in workflows
- No eval or dynamic execution of path values

---

## Documentation Requirements

### Update These Files

1. **README.md**: Add "Path Configuration" section
2. **docs/installation.md**: Document config.yml paths section
3. **docs/workflows-api.md**: NEW - Document all public workflows
4. **docs/customization.md**: How to customize paths
5. **CHANGELOG.md**: Document changes in v2.2.0

### New Documentation Files

1. **docs/workflows-api.md**: Public API documentation
2. **docs/integration-guide.md**: How to integrate with Agent-OS
3. **docs/examples/bert-integration.md**: BERT as example
4. **docs/migration-2.2.md**: Migration guide

---

## Summary of Code Changes

### Files to Modify

1. **config.yml**: Add paths section
2. **scripts/common-functions.sh**: Add path config functions
3. **scripts/project-install.sh**: Export path config
4. **workflows/specification/*.md**: Use env vars instead of hardcoded
5. **workflows/planning/*.md**: Use env vars
6. **workflows/implementation/*.md**: Use env vars
7. **.claude/commands/agent-os/*.md**: Add argument parsing for overrides

### Lines of Code Changed

- **config.yml**: +10 lines
- **common-functions.sh**: +40 lines (new functions)
- **project-install.sh**: +5 lines (call export_path_config)
- **Each workflow**: ~5-10 lines changed (replace hardcoded with vars)
- **Each command**: +30 lines (argument parsing)

**Total**: ~200-300 lines changed across all files

### Estimated Effort

- **Development**: 1-2 weeks
- **Testing**: 3-5 days
- **Documentation**: 3-5 days
- **Total**: 2-3 weeks

---

## Questions & Answers

### Q: Will this break existing projects?

**A**: No. Defaults match current hardcoded values. Zero breaking changes.

### Q: Can I mix Agent-OS defaults and custom paths?

**A**: Yes. Specify only what you want to customize, rest uses defaults.

### Q: Do I need to update all my projects?

**A**: No. Existing projects continue working. Update when you want custom paths.

### Q: Can different projects use different paths?

**A**: Yes. Each project can override paths during installation or per-command.

### Q: What if I don't want path configuration?

**A**: Don't use it. System works exactly as before with defaults.

---

**Document Version**: 1.0
**See Also**: `agent-os-integration-recommendations.md` (high-level overview)
