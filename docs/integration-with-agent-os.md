# Thin-BERT Architecture: The Real Story

**Based on actual Agent-OS v2.1 installation analysis**

---

## Critical Corrections

### 1. There is NO `.claude/agents/` Directory! ❌

Agent-OS does NOT use `.claude/agents/` - that was my mistake.

**Agent-OS uses**:
- `.claude/commands/` for Claude Code commands ONLY

**Agents are NOT separate files** - they're embedded in commands or referenced as subagents via Claude Code's native system.

---

### 2. There IS an `agent-os/` Directory in Project Root ✅

Agent-OS v2.1 **does create** an `agent-os/` directory at the project root level with:

```
your-project/
├── agent-os/
│   ├── product/
│   │   ├── mission.md
│   │   ├── roadmap.md
│   │   └── tech-stack.md
│   └── specs/
│       └── {dated-spec-name}/
│           ├── planning/
│           │   ├── requirements.md
│           │   └── visuals/
│           ├── spec.md
│           ├── tasks.md
│           └── implementation/
```

**These paths ARE hardcoded** in the workflow files!

---

### 3. Paths ARE Hardcoded in Agent-OS Workflows

From `workflows/specification/initialize-spec.md`:
```bash
SPEC_PATH="agent-os/specs/$DATED_SPEC_NAME"
```

From command output messages:
```
✅ I have initialized the spec folder at `agent-os/specs/[this-spec]`.
```

**All Agent-OS workflows expect `agent-os/` at project root.**

---

## What Agent-OS Actually Installs

### In Project Root

```
your-project/
├── .claude/
│   └── commands/
│       └── agent-os/              ← Claude Code commands
│           ├── plan-product.md
│           ├── shape-spec.md
│           ├── write-spec.md
│           ├── create-tasks.md
│           ├── implement-tasks.md
│           └── orchestrate-tasks.md
│
├── agent-os/                      ← Content directory (hardcoded paths!)
│   ├── product/
│   │   ├── mission.md
│   │   ├── roadmap.md
│   │   └── tech-stack.md
│   └── specs/
│       └── YYYY-MM-DD-spec-name/
│           ├── planning/
│           │   ├── requirements.md
│           │   └── visuals/
│           ├── spec.md
│           ├── tasks.md
│           └── implementation/
```

### If Using Standards as Claude Code Skills

```
.claude/
└── skills/
    ├── agent-os-standards-coding-style/
    ├── agent-os-standards-error-handling/
    └── ...
```

---

## How Agent-OS Commands Work

### Command Structure

Commands in `.claude/commands/agent-os/` reference:

1. **Workflows** via `{{workflows/...}}` - These get compiled/embedded during installation
2. **Subagents** via Claude Code's native subagent system (if enabled)
3. **Phases** via `{{PHASE N: @agent-os/commands/...}}` - These reference OTHER command files

Example from `plan-product.md`:
```markdown
{{PHASE 1: @agent-os/commands/plan-product/1-product-concept.md}}
{{PHASE 2: @agent-os/commands/plan-product/2-create-mission.md}}
{{PHASE 3: @agent-os/commands/plan-product/3-create-roadmap.md}}
{{PHASE 4: @agent-os/commands/plan-product/4-create-tech-stack.md}}
```

### What `@agent-os/commands/` Means

During compilation, `@agent-os/commands/X.md` references get embedded from:
- Source: `~/agent-os/profiles/default/commands/plan-product/single-agent/1-product-concept.md`
- Compiled into: `.claude/commands/agent-os/plan-product.md`

**The `@agent-os` prefix is a compilation-time reference, not a runtime directory.**

---

## The Real Thin-BERT Challenge

Given that Agent-OS has **hardcoded paths to `agent-os/`**, we have 3 options:

### Option 1: Accept `agent-os/` Directory (Simplest)

**Layered Mode Structure**:
```
your-project/
├── .claude/
│   ├── commands/
│   │   ├── agent-os/           ← Agent-OS commands (files written here)
│   │   │   ├── plan-product.md
│   │   │   ├── shape-spec.md
│   │   │   └── ...
│   │   └── bert/               ← BERT commands (thin layer)
│   │       ├── plan.md
│   │       ├── spec.md
│   │       └── task.md
│   └── skills/
│       └── bert/
│           ├── skill.yml
│           └── skill.md
│
├── agent-os/                   ← Agent-OS content (hardcoded paths)
│   ├── product/
│   │   ├── mission.md
│   │   ├── roadmap.md
│   │   └── tech-stack.md
│   └── specs/
│       └── {specs}/
│
└── docs/bert/                  ← BERT-specific additions
    ├── tasks/                  ← Task files (NOT in agent-os)
    │   ├── task-01.1.md
    │   ├── task-01.2.md
    │   └── task-01-review.md   ← Review files (BERT's feature)
    └── archive/                ← Archive (BERT's feature)
```

**BERT delegates to Agent-OS commands**:
- `/bert:plan` → calls `/plan-product` (which writes to `agent-os/product/`)
- `/bert:spec new` → calls `/shape-spec` (which writes to `agent-os/specs/`)
- `/bert:spec tasks` → calls `/create-tasks` (which writes to `agent-os/specs/{spec}/tasks.md`)
- `/bert:task execute` → calls `/implement-tasks` + generates review files in `docs/bert/tasks/`

**Pros**:
- No path translation needed
- Agent-OS works as-is
- BERT just adds review workflow on top

**Cons**:
- Two content directories: `agent-os/` and `docs/bert/tasks/`
- Not as clean as single directory

---

### Option 2: Symlink Approach

```
your-project/
├── .claude/commands/
│   ├── agent-os/           ← Agent-OS commands
│   └── bert/               ← BERT commands
│
├── docs/bert/              ← Primary location
│   ├── product/
│   ├── specs/
│   ├── tasks/
│   └── archive/
│
└── agent-os/               ← Symlink to docs/bert
    ├── product -> ../docs/bert/product
    └── specs -> ../docs/bert/specs
```

**Pros**:
- Content appears in both places
- Agent-OS commands work (write to `agent-os/*`)
- BERT has clean `docs/bert/` structure
- Single source of truth

**Cons**:
- Symlinks can be confusing
- Git behavior with symlinks varies
- Windows compatibility issues

---

### Option 3: Path Translation in BERT Layer

**BERT commands do path translation before delegating**:

```markdown
# .claude/commands/bert/spec.md

## Subcommand: new

1. Create spec in docs/bert/specs/{spec-nn}/
2. Call Agent-OS shape-spec with path override:
   - Read from docs/bert/specs/{spec-nn}/
   - BUT tell Agent-OS it's agent-os/specs/{spec-nn}/
   - Agent-OS commands expect agent-os/ paths in their output messages
```

**Pros**:
- BERT has clean directory structure
- No symlinks

**Cons**:
- Complex translation logic
- Agent-OS output messages say "agent-os/" but files are in "docs/bert/"
- Confusing for users
- Fragile

---

## Recommended Approach: Option 1 (Accept agent-os/ Directory)

**Why**: Simplest, most reliable, respects Agent-OS's architecture

### Final Structure (Layered Mode)

```
your-project/
├── .claude/
│   ├── commands/
│   │   ├── agent-os/              ← Agent-OS commands
│   │   │   ├── plan-product.md
│   │   │   ├── shape-spec.md
│   │   │   ├── write-spec.md
│   │   │   ├── create-tasks.md
│   │   │   ├── implement-tasks.md
│   │   │   └── orchestrate-tasks.md
│   │   │
│   │   └── bert/                  ← BERT thin layer
│   │       ├── plan.md            (~15 lines - delegates to /plan-product)
│   │       ├── spec.md            (~100 lines - smart router)
│   │       └── task.md            (~150 lines - task CRUD + review workflow)
│   │
│   └── skills/
│       ├── agent-os-standards-*/  ← If using standards as Skills
│       └── bert/
│           ├── skill.yml
│           └── skill.md
│
├── agent-os/                      ← Agent-OS content (spec, product)
│   ├── product/
│   │   ├── mission.md
│   │   ├── roadmap.md
│   │   └── tech-stack.md
│   └── specs/
│       └── YYYY-MM-DD-spec-name/
│           ├── planning/
│           │   ├── requirements.md
│           │   └── visuals/
│           ├── spec.md
│           └── tasks.md           ← Single tasks.md file
│
└── docs/bert/                     ← BERT additions
    ├── tasks/                     ← Individual task files
    │   ├── task-01.1-database.md     (split from agent-os tasks.md)
    │   ├── task-01.2-api.md
    │   └── task-01-review.md         (BERT's review file)
    └── archive/                   ← Archive (BERT feature)
        ├── specs/
        └── tasks/
```

---

## How This Works

### /bert:plan
```markdown
# .claude/commands/bert/plan.md

Call Agent-OS's plan-product command:

Delegation: Run /plan-product

This will create:
- agent-os/product/mission.md
- agent-os/product/roadmap.md
- agent-os/product/tech-stack.md

Output BERT-style message:
"✓ Product planning complete! Files in agent-os/product/"
```

---

### /bert:spec new
```markdown
# .claude/commands/bert/spec.md (new subcommand)

Call Agent-OS's shape-spec command:

Delegation: Run /shape-spec with description "{description}"

This will create:
- agent-os/specs/{dated-name}/planning/requirements.md

Output BERT-style message:
"✓ Created spec at agent-os/specs/{name}/
Fill in requirements.md, then: /bert:spec iterate {spec-number}"
```

---

### /bert:spec iterate
```markdown
# .claude/commands/bert/spec.md (iterate subcommand)

Smart routing logic:

1. Detect state by checking files in agent-os/specs/{spec}/
   - Has requirements.md? Has spec.md? Has feedback?

2. Based on state, call appropriate Agent-OS command:
   - IF needs follow-ups: /shape-spec (phase 2)
   - IF ready for spec: /write-spec
   - IF has feedback: /write-spec (with feedback mode)

All work happens in agent-os/specs/{spec}/ (Agent-OS's territory)
```

---

### /bert:spec tasks
```markdown
# .claude/commands/bert/spec.md (tasks subcommand)

1. Call Agent-OS's create-tasks command:
   Delegation: Run /create-tasks for spec {number}

   This creates: agent-os/specs/{spec}/tasks.md

2. Read agent-os/specs/{spec}/tasks.md

3. Split into individual task files in docs/bert/tasks/:
   - task-01.1-database.md
   - task-01.2-api.md
   - task-01.3-frontend.md

4. Output:
   "✓ Created 3 task files in docs/bert/tasks/
   Execute with: /bert:task execute 01.1"
```

---

### /bert:task execute
```markdown
# .claude/commands/bert/task.md (execute subcommand)

Arguments: Task numbers (e.g., "01.1,01.2,01.3")

1. Read task files from docs/bert/tasks/task-{nn}.{sub}.md

2. For each task, call Agent-OS's implement-tasks:
   Delegation: Run /implement-tasks with task content

   Agent-OS does implementation (files changed tracked)

3. After all tasks done, generate review file:
   Create docs/bert/tasks/task-01-review.md with:
   - Tasks completed
   - Files changed
   - Testing checklist
   - Issues section (empty, for user)

4. Output:
   "✓ Tasks complete! Review file: docs/bert/tasks/task-01-review.md
   Test and add issues, then: 'added notes to task-01-review.md'"
```

---

## BERT's Additions (What BERT Adds to Agent-OS)

### 1. Smart Spec Iteration
**Agent-OS has**: `/shape-spec` and `/write-spec` as separate commands
**BERT adds**: `/bert:spec iterate` that auto-detects which to call

### 2. Individual Task Files
**Agent-OS has**: Single `tasks.md` file per spec
**BERT adds**: Individual task files in `docs/bert/tasks/` (split from tasks.md)

### 3. Review Workflow
**Agent-OS has**: Verification reports (formal, automated)
**BERT adds**: Review files (living documents, user-driven issue tracking)

### 4. Task Management
**Agent-OS has**: No standalone task system
**BERT adds**: Ad-hoc tasks, hierarchical numbering, task CRUD operations

### 5. Simpler Commands
**Agent-OS has**: 6 phase-based commands
**BERT adds**: 3 namespaced commands with smart routing

---

## Installation Scripts (Corrected)

### Standalone BERT
```bash
#!/bin/bash
# install.sh - BERT standalone (NO Agent-OS)

set -e
echo "Installing BERT (standalone)..."

# Create directories
mkdir -p .claude/commands/bert
mkdir -p .claude/skills/bert
mkdir -p docs/bert/{product,specs,tasks,archive}

# Download BERT commands (self-sufficient, includes embedded workflows)
curl -fsSL .../bert/plan.md -o .claude/commands/bert/plan.md
curl -fsSL .../bert/spec.md -o .claude/commands/bert/spec.md
curl -fsSL .../bert/task.md -o .claude/commands/bert/task.md

# Download skill config
curl -fsSL .../bert/skill.yml -o .claude/skills/bert/skill.yml
curl -fsSL .../bert/skill.md -o .claude/skills/bert/skill.md

echo "✓ BERT installed! Commands: /bert:plan, /bert:spec, /bert:task"
echo "Content will be in docs/bert/"
```

---

### Layered BERT (Over Agent-OS)
```bash
#!/bin/bash
# install-over-agent-os.sh

set -e

# Check if Agent-OS is installed
if [ ! -d ".claude/commands/agent-os" ]; then
    echo "Error: Agent-OS not found!"
    echo "Install Agent-OS first: ~/agent-os/scripts/project-install.sh"
    exit 1
fi

echo "Installing BERT layer over Agent-OS..."

# Create BERT directories (NOT agent-os - that exists!)
mkdir -p .claude/commands/bert
mkdir -p .claude/skills/bert
mkdir -p docs/bert/tasks         # For individual task files
mkdir -p docs/bert/archive       # For archiving

# Download BERT commands (thin, delegate to Agent-OS)
curl -fsSL .../bert-layered/plan.md -o .claude/commands/bert/plan.md
curl -fsSL .../bert-layered/spec.md -o .claude/commands/bert/spec.md
curl -fsSL .../bert-layered/task.md -o .claude/commands/bert/task.md

# Download skill config
curl -fsSL .../bert/skill.yml -o .claude/skills/bert/skill.yml
curl -fsSL .../bert/skill.md -o .claude/skills/bert/skill.md

echo ""
echo "✓ BERT installed over Agent-OS!"
echo ""
echo "You have both interfaces:"
echo "  BERT: /bert:plan, /bert:spec, /bert:task"
echo "  Agent-OS: /plan-product, /shape-spec, /write-spec, etc."
echo ""
echo "Content locations:"
echo "  agent-os/product/ - Product context (both systems)"
echo "  agent-os/specs/   - Specs (both systems)"
echo "  docs/bert/tasks/  - Task files (BERT only)"
echo ""
echo "BERT delegates to Agent-OS for all heavy lifting."
```

---

## File Count & Code Size

### Standalone BERT
- Commands: 3 files (~400 lines with embedded workflows)
- Skills: 2 files (~100 lines)
**Total**: 5 files, ~500 lines

### Layered BERT
- Commands: 3 files (~265 lines - thin, delegates to Agent-OS)
- Skills: 2 files (~100 lines)
**Total**: 5 files, ~365 lines

**Savings in layered mode**: ~135 lines (27% smaller!)

---

## Key Realizations

### 1. Agent-OS Uses `.claude/commands/` ONLY

There is no `.claude/agents/` directory. Agents are either:
- Embedded in commands
- Or delegated via Claude Code's native subagent system (if enabled)

### 2. `agent-os/` Directory is Required

Paths like `agent-os/specs/` and `agent-os/product/` are **hardcoded** in Agent-OS workflows. We must accept this.

### 3. BERT Adds Its Own Directory

`docs/bert/tasks/` for:
- Individual task files (split from Agent-OS's tasks.md)
- Review files (BERT's unique feature)
- Archive system

### 4. Layered BERT is Actually Simpler

Because Agent-OS does the heavy lifting, BERT layer is truly thin:
- No embedded workflows needed
- Just delegation + review file generation
- ~365 lines total

---

## The True Value Proposition

**Agent-OS** = Comprehensive spec-driven development engine
- Product planning
- Spec creation (requirements → spec)
- Task breakdown
- Implementation with standards
- Files in `agent-os/` directory

**BERT** = UI layer + developer workflow enhancements
- Simple commands (`/bert:spec iterate` vs `/shape-spec` + `/write-spec`)
- Individual task files (vs single tasks.md)
- Review workflow (vs verification reports)
- Ad-hoc task management
- Files in `docs/bert/tasks/`

**Together**: Agent-OS is the API, BERT is the UI

---

## User Journey (Corrected)

```bash
# Install Agent-OS
~/agent-os/scripts/project-install.sh

# Add BERT layer
curl ... install-over-agent-os.sh | bash

# Use BERT interface
/bert:spec new "authentication"
# → Delegates to /shape-spec
# → Creates agent-os/specs/YYYY-MM-DD-authentication/

[Fill in agent-os/specs/.../planning/requirements.md]

/bert:spec iterate
# → Detects state, calls /write-spec
# → Creates agent-os/specs/.../spec.md

/bert:spec tasks
# → Calls /create-tasks
# → Creates agent-os/specs/.../tasks.md
# → Splits into docs/bert/tasks/task-*.md files

/bert:task execute 01.1,01.2
# → Calls /implement-tasks twice
# → Generates docs/bert/tasks/task-01-review.md

[Test, add issues to review file]

"added notes to task-01-review.md"
# → BERT reads issues, calls /implement-tasks for fixes
# → Updates review file with fixes

# Can also use Agent-OS directly!
/shape-spec    # More control
/write-spec    # Step-by-step
/create-tasks  # Explicit
```

**Result**: Best of both worlds!
