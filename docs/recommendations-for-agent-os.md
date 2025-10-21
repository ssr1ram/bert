# Recommendations for Agent-OS: Enabling Better Integration

**To**: Brian Casel, Builder Methods (Agent-OS maintainer)
**From**: BERT Development Team
**Date**: 2025-10-21
**Subject**: Making Agent-OS More Integrable for Tools Like BERT

---

## Executive Summary

We've been building BERT, a developer-friendly task management tool that could benefit greatly from integrating with Agent-OS. During our analysis, we identified several architectural improvements that would make Agent-OS easier to integrate with, extend, and build upon - not just for BERT, but for any tool in the ecosystem.

**Key recommendations**:
1. **Make paths configurable** (not hardcoded)
2. **Provide a clear API layer** (workflows as reusable modules)
3. **Support path overrides** in commands
4. **Standardize file reference patterns**
5. **Enable partial adoption** (use only what you need)

These changes would position Agent-OS as a **platform** that other tools can build on, rather than just a standalone system.

---

## Current State: What Makes Integration Difficult

### 1. Hardcoded Paths in Workflows

**Current implementation** (`workflows/specification/initialize-spec.md`):
```bash
SPEC_PATH="agent-os/specs/$DATED_SPEC_NAME"
mkdir -p $SPEC_PATH/planning
```

**Problem**:
- Any tool integrating with Agent-OS MUST use `agent-os/` directory
- Cannot customize paths to fit their project structure
- Forces specific directory naming on users

**Impact on BERT**:
- Want to use `docs/bert/specs/` but can't
- Must accept `agent-os/` directory even in "BERT mode"
- Creates confusion (two systems, two directories)

---

### 2. Output Messages Reference Hardcoded Paths

**Current implementation**:
```markdown
✅ I have initialized the spec folder at `agent-os/specs/[this-spec]`.
```

**Problem**:
- User sees "agent-os/specs/" even if files are elsewhere
- Messages don't reflect actual file locations if paths are customized

---

### 3. No Clear API Boundary

**Current structure**:
```
profiles/default/
├── commands/        # User-facing commands
├── agents/          # Subagents
└── workflows/       # Reusable logic (but tightly coupled to commands)
```

**Problem**:
- Workflows are embedded/referenced by commands (not standalone)
- No clear "public API" vs "internal implementation"
- Hard to know what's safe to call from external tools

**What we need**:
- Clear distinction: "These workflows are public APIs"
- Documentation: "Call these workflows from your tools"
- Stability guarantees: "These APIs won't break between versions"

---

### 4. Configuration is Project-Level Only

**Current**: `config.yml` in base installation, no per-command overrides

**Problem**:
- Can't override paths per-command
- All commands use same configuration
- Hard to mix Agent-OS defaults with custom paths

---

## Recommendations

### Recommendation 1: Make Paths Configurable

**Priority**: HIGH
**Effort**: Medium
**Impact**: Massive - enables all integrations

#### Proposed Solution

Add a configuration system for path templates:

```yaml
# config.yml
version: 2.2.0

paths:
  specs_directory: "agent-os/specs"
  specs_naming: "{date}-{name}"        # Template: YYYY-MM-DD-spec-name
  product_directory: "agent-os/product"
  tasks_file: "tasks.md"               # Within spec directory
  implementation_directory: "implementation"

# Allow overrides
path_overrides:
  # Tools can override these
  specs_directory_override: null       # null = use default
  product_directory_override: null
```

#### Update Workflows to Use Config

**Before** (`workflows/specification/initialize-spec.md`):
```bash
SPEC_PATH="agent-os/specs/$DATED_SPEC_NAME"
```

**After**:
```bash
# Read from config or environment
SPECS_DIR="${SPECS_DIRECTORY:-agent-os/specs}"
SPEC_NAMING="${SPEC_NAMING_TEMPLATE:-{date}-{name}}"

# Build path from template
DATED_SPEC_NAME=$(echo "$SPEC_NAMING" | sed "s/{date}/$TODAY/" | sed "s/{name}/$SPEC_NAME/")
SPEC_PATH="$SPECS_DIR/$DATED_SPEC_NAME"
```

#### Pass Paths as Parameters

**Commands receive paths as environment variables**:

```bash
# When calling workflow
export SPECS_DIRECTORY="docs/bert/specs"
export PRODUCT_DIRECTORY="docs/bert/product"

{{workflows/specification/initialize-spec}}
# Workflow uses $SPECS_DIRECTORY instead of hardcoded path
```

#### Benefits

- **BERT can use**: `docs/bert/specs/`
- **Agent-OS defaults to**: `agent-os/specs/`
- **Other tools can choose**: Whatever fits their structure
- **Zero breaking changes**: Default values match current behavior

---

### Recommendation 2: Provide Workflow Library as Public API

**Priority**: HIGH
**Effort**: Low (documentation + minor refactoring)
**Impact**: Enables ecosystem growth

#### Current Problem

Workflows exist but aren't documented as reusable APIs:
- `workflows/specification/initialize-spec.md` - could be reused
- `workflows/specification/write-spec.md` - could be reused
- But no documentation on HOW to call them externally

#### Proposed Solution

**Create a "Workflows API" documentation**:

```markdown
# Agent-OS Workflows API

## Public Workflows (Stable API)

These workflows are designed to be called by external tools and will maintain backward compatibility.

### Specification Workflows

#### `workflows/specification/initialize-spec`
**Purpose**: Create a new spec folder structure
**Inputs** (via environment variables):
  - SPECS_DIRECTORY: Where to create spec (default: agent-os/specs)
  - SPEC_NAME: Kebab-case spec name
  - SPEC_NAMING_TEMPLATE: How to name folders (default: {date}-{name})
**Outputs**:
  - Creates folder: $SPECS_DIRECTORY/$DATED_SPEC_NAME
  - Returns: Path to created spec
**Example**:
  ```bash
  export SPECS_DIRECTORY="docs/myapp/specs"
  export SPEC_NAME="authentication"
  {{workflows/specification/initialize-spec}}
  ```

#### `workflows/specification/write-spec`
**Purpose**: Generate spec.md from requirements
**Inputs**:
  - SPEC_PATH: Path to spec directory
  - (reads $SPEC_PATH/planning/requirements.md)
**Outputs**:
  - Creates: $SPEC_PATH/spec.md
**Example**:
  ```bash
  export SPEC_PATH="docs/myapp/specs/2025-10-21-auth"
  {{workflows/specification/write-spec}}
  ```

### Implementation Workflows

#### `workflows/implementation/implement-tasks`
**Purpose**: Implement tasks from a task list
**Inputs**:
  - SPEC_PATH: Path to spec directory
  - TASK_NUMBERS: Which tasks to implement (optional, defaults to all)
  - SKIP_VERIFICATION: Skip verification phase (default: false)
**Outputs**:
  - Implements tasks
  - Returns: List of files changed
**Example**:
  ```bash
  export SPEC_PATH="docs/myapp/specs/2025-10-21-auth"
  export TASK_NUMBERS="1,2,3"
  export SKIP_VERIFICATION="true"
  {{workflows/implementation/implement-tasks}}
  ```
```

#### Version Each Workflow API

Add version markers to workflows:
```markdown
<!-- Agent-OS Workflow API v1.0 -->
<!-- Stability: Stable - Breaking changes will increment major version -->

# Spec Initialization
...
```

#### Benefits

- **Clear contract**: Tools know what they can depend on
- **Stability**: API versioning prevents breakage
- **Ecosystem**: Other tools can build on Agent-OS
- **Documentation**: Easy to understand what's available

---

### Recommendation 3: Support Path Overrides in Commands

**Priority**: MEDIUM
**Effort**: Medium
**Impact**: Enables flexible integration

#### Proposed Solution

Commands accept path overrides as flags:

```markdown
# .claude/commands/agent-os/shape-spec.md

Parse optional flags:
--specs-dir PATH       Override specs directory (default: from config.yml)
--product-dir PATH     Override product directory (default: from config.yml)
--naming-template TPL  Override spec naming (default: {date}-{name})

Example:
/shape-spec "authentication" --specs-dir docs/myapp/specs
```

**Implementation**:
```markdown
# shape-spec.md

Parse arguments:
if args contain "--specs-dir":
  export SPECS_DIRECTORY="{value}"
else:
  export SPECS_DIRECTORY=$(get_config "paths.specs_directory" "agent-os/specs")

# Pass to workflow
{{workflows/specification/initialize-spec}}
```

#### Benefits

- **Flexibility**: Override paths per-command
- **Backward compatible**: Flags are optional
- **BERT can do**: `/shape-spec "auth" --specs-dir docs/bert/specs`
- **Agent-OS users**: Continue using defaults

---

### Recommendation 4: Standardize File Reference Patterns

**Priority**: LOW
**Effort**: Low
**Impact**: Improves maintainability

#### Current Issues

Multiple reference patterns:
- `{{workflows/specification/initialize-spec}}` - workflow
- `{{PHASE 1: @agent-os/commands/plan-product/1-product-concept.md}}` - phase
- `@agent-os/specs/...` - file path (but only at compile time)

Not clear which is which or how they're resolved.

#### Proposed Solution

**Standardize and document**:

```markdown
# Agent-OS Reference Patterns

## Workflow References
Format: {{workflows/category/workflow-name}}
Example: {{workflows/specification/initialize-spec}}
Resolved: Embedded during compilation from profiles/default/workflows/

## Phase References (Internal)
Format: {{PHASE N: @agent-os/commands/path}}
Example: {{PHASE 1: @agent-os/commands/plan-product/1-product-concept.md}}
Resolved: Embedded from profiles/default/commands/

## File Path Variables (Runtime)
Format: $VARIABLE_NAME
Example: $SPEC_PATH/planning/requirements.md
Resolved: From environment variables or config

## Conditional Blocks
Format: {{IF flag}} ... {{ENDIF flag}}
Example: {{IF use_claude_code_subagents}} ... {{ENDIF}}
Resolved: During compilation based on config flags
```

#### Benefits

- **Clarity**: Developers know what each pattern means
- **Consistency**: Easier to maintain
- **External tools**: Know how to reference workflows

---

### Recommendation 5: Enable Partial Adoption

**Priority**: MEDIUM
**Effort**: Low (documentation)
**Impact**: Lowers barrier to entry

#### Current Problem

Agent-OS feels "all or nothing":
- Install all 6 commands or use none
- Use full workflow or build from scratch
- No guidance on "just use spec writing" or "just use implementation"

#### Proposed Solution

**Document modular usage patterns**:

```markdown
# Agent-OS Modular Usage

You don't need to use all of Agent-OS. Here's how to use specific parts:

## Use Case 1: Just Spec Writing

Install only spec-related workflows:
1. Copy `workflows/specification/` to your project
2. Call `{{workflows/specification/write-spec}}` when you need a spec
3. Skip product planning, tasks, implementation

## Use Case 2: Just Implementation

Install only implementation workflows:
1. Copy `workflows/implementation/` to your project
2. Create your own task system
3. Call `{{workflows/implementation/implement-tasks}}` for execution

## Use Case 3: Spec + Custom Task Management (BERT's approach)

1. Use Agent-OS for specs: `/shape-spec`, `/write-spec`
2. Build custom task system on top
3. Call Agent-OS implementer when ready to code

## Use Case 4: Full Agent-OS

Use everything as designed!
```

#### Create "Integration Examples"

```markdown
# Integration Examples

## Example: Building a Tool Like BERT

BERT wants:
- Agent-OS's spec writing (requirements → spec.md)
- Agent-OS's implementation engine
- But custom task management and review workflow

How BERT integrates:
1. Install Agent-OS with `--specs-dir docs/bert/specs`
2. Call `/shape-spec` and `/write-spec` for specs
3. Build custom `/bert:task` system
4. Call Agent-OS implementer via workflows API
5. Add custom review file generation

Result: Best of both worlds!
```

#### Benefits

- **Lower barrier**: "You can use just the parts you need"
- **Clearer**: Users understand what each piece does
- **Ecosystem**: Encourages building tools on Agent-OS

---

### Recommendation 6: Expose Configuration to Claude Code Skills

**Priority**: LOW
**Effort**: Medium
**Impact**: Better Claude Code integration

#### Proposed Solution

If using Claude Code Skills for standards, also provide a **config skill**:

```yaml
# .claude/skills/agent-os-config/skill.yml
name: agent-os-config
description: Agent-OS configuration and paths
model: inherit
```

```markdown
# .claude/skills/agent-os-config/skill.md

Agent-OS is configured as follows:

**Paths**:
- Specs: ${SPECS_DIRECTORY}
- Product: ${PRODUCT_DIRECTORY}
- Naming: ${SPEC_NAMING_TEMPLATE}

**Features**:
- Use subagents: ${USE_CLAUDE_CODE_SUBAGENTS}
- Standards as Skills: ${STANDARDS_AS_CLAUDE_CODE_SKILLS}

When working on this project, use these paths for Agent-OS operations.
```

#### Benefits

- **Agents know config**: Can reference correct paths
- **Consistent**: All commands use same config
- **Discoverable**: Claude Code can see config as a Skill

---

## Implementation Roadmap

### Phase 1: Path Configuration (v2.2.0)

**Goal**: Make paths configurable without breaking existing users

**Tasks**:
1. Add `paths` section to `config.yml`
2. Update workflows to read from environment variables
3. Add fallbacks to hardcoded defaults (backward compatible)
4. Update installation script to export paths as env vars
5. Document new path configuration

**Effort**: 2-3 weeks
**Breaking changes**: None (defaults match current behavior)

---

### Phase 2: Workflows API Documentation (v2.2.0 or v2.3.0)

**Goal**: Document workflows as public APIs

**Tasks**:
1. Create `docs/workflows-api.md`
2. Document each public workflow (inputs, outputs, examples)
3. Add version markers to workflows
4. Add "Integration Guide" with examples

**Effort**: 1 week
**Breaking changes**: None (documentation only)

---

### Phase 3: Command Path Overrides (v2.3.0)

**Goal**: Allow per-command path customization

**Tasks**:
1. Add argument parsing for `--specs-dir`, `--product-dir` flags
2. Update commands to accept overrides
3. Pass overrides to workflows as env vars
4. Document flag usage

**Effort**: 1-2 weeks
**Breaking changes**: None (flags are optional)

---

### Phase 4: Modular Usage Guide (v2.3.0)

**Goal**: Document how to use parts of Agent-OS

**Tasks**:
1. Create `docs/modular-usage.md`
2. Document use cases (specs only, implementation only, etc.)
3. Add integration examples (BERT, others)
4. Update README with "You don't need to use everything"

**Effort**: 3-5 days
**Breaking changes**: None (documentation only)

---

## Specific Code Changes

### Example 1: Make initialize-spec.md Configurable

**Before**:
```bash
SPEC_PATH="agent-os/specs/$DATED_SPEC_NAME"
mkdir -p $SPEC_PATH/planning
mkdir -p $SPEC_PATH/planning/visuals
```

**After**:
```bash
# Read from config or environment (with fallback)
SPECS_DIR="${SPECS_DIRECTORY:-agent-os/specs}"
SPEC_NAMING="${SPEC_NAMING_TEMPLATE:-{date}-{name}}"

# Build spec name from template
SPEC_NAME="[kebab-case-name]"
TODAY=$(date +%Y-%m-%d)
DATED_SPEC_NAME=$(echo "$SPEC_NAMING" | sed "s/{date}/$TODAY/" | sed "s/{name}/$SPEC_NAME/")

# Build full path
SPEC_PATH="$SPECS_DIR/$DATED_SPEC_NAME"

# Create folders
mkdir -p "$SPEC_PATH/planning"
mkdir -p "$SPEC_PATH/planning/visuals"
```

**Output message**:
```markdown
✅ I have initialized the spec folder at `${SPEC_PATH}`.
```

Now outputs actual path, not hardcoded "agent-os/specs/"!

---

### Example 2: Export Paths in Installation

**In `project-install.sh`**:

```bash
# Read paths from config
SPECS_DIR=$(get_yaml_value "$BASE_DIR/config.yml" "paths.specs_directory" "agent-os/specs")
PRODUCT_DIR=$(get_yaml_value "$BASE_DIR/config.yml" "paths.product_directory" "agent-os/product")
SPEC_NAMING=$(get_yaml_value "$BASE_DIR/config.yml" "paths.specs_naming" "{date}-{name}")

# When compiling commands, inject these as environment defaults
compile_command() {
    local source=$1
    local dest=$2

    # Add env var exports at top of compiled command
    {
        echo "# Agent-OS Configuration"
        echo "SPECS_DIRECTORY=\"${SPECS_DIR}\""
        echo "PRODUCT_DIRECTORY=\"${PRODUCT_DIR}\""
        echo "SPEC_NAMING_TEMPLATE=\"${SPEC_NAMING}\""
        echo ""
        cat "$source"
    } > "$dest"
}
```

---

### Example 3: Add Path Overrides to Commands

**In `.claude/commands/agent-os/shape-spec.md`**:

```markdown
You are helping to shape a specification.

## Parse Arguments

Check if arguments include path overrides:

```bash
# Initialize with defaults or config
SPECS_DIR="${SPECS_DIRECTORY:-agent-os/specs}"
PRODUCT_DIR="${PRODUCT_DIRECTORY:-agent-os/product}"

# Parse command line args for overrides
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
    *)
      DESCRIPTION="$1"
      shift
      ;;
  esac
done

# Export for workflows to use
export SPECS_DIRECTORY="$SPECS_DIR"
export PRODUCT_DIRECTORY="$PRODUCT_DIR"
```

Now call workflows:

{{PHASE 1: @agent-os/commands/shape-spec/1-initialize-spec.md}}

Workflows will use $SPECS_DIRECTORY instead of hardcoded path.
```

---

## Benefits to Agent-OS

### 1. Becomes a Platform

Current: Agent-OS is a standalone system
Future: **Agent-OS is a platform** others build on

Example ecosystem:
- BERT: Task management + review workflow built on Agent-OS
- Spec-Only: Tool that just uses Agent-OS spec generation
- Custom Implementer: Uses Agent-OS specs but custom implementation
- Enterprise: Agent-OS + custom standards + custom workflows

### 2. More Adoption

**Barrier to entry lowers**:
- "I like your spec system but not task breakdown" → Use just specs!
- "I need custom paths" → Configure them!
- "I want to extend Agent-OS" → Here's the API!

**Result**: More users, more contributors, more ecosystem growth

### 3. Better Testing & Maintenance

**Clear API boundaries**:
- Public workflows (stable, versioned)
- Internal implementation (can refactor freely)
- Tests focus on API contracts

**Easier to maintain**:
- Know what can't break (public APIs)
- Know what can change (internal implementation)

### 4. Competitive Differentiation

**Instead of**: "Agent-OS vs BERT vs Other Tools"
**Becomes**: "Agent-OS powers BERT, powers Other Tools"

**Positioning**:
- Agent-OS = The engine
- BERT = Developer-friendly UI
- Others = Custom integrations

**Winner**: Agent-OS becomes the standard platform

---

## Compatibility Strategy

### All Changes Are Backward Compatible

**Principle**: Don't break existing users

1. **Path config**:
   - Add to config.yml (optional)
   - Defaults match current hardcoded values
   - Existing users see no change

2. **Workflow APIs**:
   - Documentation only
   - No code changes to workflows initially
   - Just make explicit what's already there

3. **Command overrides**:
   - New flags are optional
   - No flags = current behavior
   - Existing users unaffected

4. **Modular usage**:
   - Documentation of existing capabilities
   - No code changes needed

**Result**: Zero breaking changes, only additions

---

## Request for Feedback

We'd love to collaborate on this! Specifically:

### Questions for Agent-OS Team

1. **Path configuration**: Is the proposed `config.yml` approach aligned with your vision?

2. **Workflows API**: Would you consider workflows as a "public API" with version stability?

3. **Integration examples**: Would you welcome PRs that add BERT as an official integration example?

4. **Roadmap alignment**: Do these changes fit with your plans for Agent-OS v2.x?

5. **Breaking changes**: Any concerns about backward compatibility with the proposed changes?

---

## Potential Collaboration

### What BERT Can Contribute

1. **First integration example**: BERT as a reference implementation

2. **Testing**: We'll test configurable paths extensively

3. **Documentation**: Help document workflows API (we've analyzed them deeply)

4. **Use cases**: Real-world feedback on what works/doesn't

5. **Code contributions**: We can submit PRs for path configuration

### What We Need from Agent-OS

1. **Path configuration system**: Core requirement for clean integration

2. **Stable workflow APIs**: So BERT doesn't break with Agent-OS updates

3. **Documentation**: Clear contracts for what we can depend on

4. **Communication**: Heads up on breaking changes

---

## Conclusion

Agent-OS has incredible potential as a **platform** for spec-driven development tools. With relatively minor changes - primarily making paths configurable and documenting workflows as APIs - Agent-OS could become the foundation that many tools build upon.

**The vision**:
- Agent-OS = The powerful engine (specs, implementation, standards)
- BERT = The simple UI (task management, review workflow)
- Other tools = Custom integrations for specific needs

**All powered by Agent-OS workflows.**

We believe these changes benefit not just BERT, but the entire ecosystem. They position Agent-OS as the **standard platform** for AI-powered development workflows.

We'd love to discuss these ideas and explore how BERT and Agent-OS can work together!

---

## Appendix: Comparison Matrix

| Feature | Current Agent-OS | With Recommendations | BERT Standalone | BERT + Agent-OS |
|---------|-----------------|---------------------|-----------------|-----------------|
| **Paths** | Hardcoded `agent-os/` | Configurable | Uses `docs/bert/` | Uses `docs/bert/` |
| **Spec system** | Full featured | Full featured | Basic | Full (from Agent-OS) |
| **Task system** | tasks.md file | tasks.md file | Individual files | Individual files |
| **Implementation** | Via commands | Via commands | Basic | Full (from Agent-OS) |
| **Standards** | Full system | Full system | None | Full (from Agent-OS) |
| **Review workflow** | Verification | Verification | Review files | Review files |
| **Modular** | All-or-nothing | Pick and choose | Built-in | Pick and choose |
| **Integration** | Difficult | Easy (APIs) | N/A | Clean (via APIs) |

**Sweet spot**: BERT + Agent-OS with recommended changes = Best of both worlds

---

## Contact

For discussion:
- GitHub: [Create issue on BERT repo with feedback]
- Email: [Your preferred contact]
- Or via Builder Methods Pro community

We're excited about the potential collaboration!

---

**Document Version**: 1.0
**Date**: 2025-10-21
**Status**: Proposal for Discussion
