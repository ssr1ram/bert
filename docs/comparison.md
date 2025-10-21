# BERT vs Agent-OS: Detailed Comparison

**BERT Version**: 0.1.2
**Agent-OS Version Compared**: 2.1.0 (Current)
**BERT Started From**: Agent-OS v2.0.1

---

## Executive Summary

BERT is a specialized fork of Agent-OS designed for **developer-centric task execution with built-in review workflows**. While Agent-OS provides a comprehensive spec-driven development system with multiple phases and extensive customization, BERT simplifies and focuses on:

1. **Task-first workflow** with hierarchical task management
2. **Built-in review and iteration cycles** with auto-generated review files
3. **Simplified command structure** (3 commands vs 6+ phases)
4. **Async issue tracking** directly in review files
5. **Unified interface** with namespaced commands (`/bert:task`, `/bert:spec`, `/bert:plan`)

---

## Architecture Comparison

### Command Structure

| Aspect | Agent-OS 2.1 | BERT 0.1.2 | Winner |
|--------|--------------|------------|--------|
| **Number of Commands** | 6 main phases | 3 namespaced commands | **BERT** (Simpler) |
| **Command Organization** | Phase-based (`/plan-product`, `/shape-spec`, `/write-spec`, `/create-tasks`, `/implement-tasks`, `/orchestrate-tasks`) | Feature-based (`/bert:plan`, `/bert:spec`, `/bert:task`) | **BERT** (More intuitive) |
| **Command Naming** | Descriptive actions | Namespaced with resource types | **BERT** (Better organization) |
| **Single/Multi-agent modes** | Config-driven (claude_code_commands, use_claude_code_subagents, agent_os_commands) | Built-in with optional subagents | **Tie** (Different approaches) |

### Workflow Philosophy

| Aspect | Agent-OS 2.1 | BERT 0.1.2 | Winner |
|--------|--------------|------------|--------|
| **Primary Focus** | Spec-driven development with standards compliance | Task execution with review cycles | **Depends on use case** |
| **User Control** | Multiple distinct phases, pick what you need | Streamlined workflows, optional spec layer | **BERT** (Faster iteration) |
| **Flexibility** | High - 6 phases, mix and match | Medium - 3 workflows, all integrated | **Agent-OS** (More options) |
| **Learning Curve** | Steeper (6 phases, configuration options) | Gentler (3 commands, clear workflows) | **BERT** (Easier to learn) |

---

## Feature-by-Feature Comparison

### 1. Product Planning

#### Agent-OS (`/plan-product`)
- Creates: `mission.md`, `roadmap.md`, `tech-stack.md`
- Location: `agent-os/product/`
- Workflow: Single command, product-planner agent
- Standards integration: Can reference in later phases

#### BERT (`/bert:plan`)
- Creates: `mission.md`, `roadmap.md`, `tech-stack.md`
- Location: `docs/bert/product/`
- Workflow: Single command, creates templates
- Optional: Can skip entirely for immediate coding

**Winner**: **Tie** - Nearly identical functionality

---

### 2. Specification Development

#### Agent-OS
**Commands**: `/shape-spec`, `/write-spec`, `/create-tasks`

**Workflow**:
1. `/shape-spec` - Initialize requirements.md, gather requirements
2. `/write-spec` - Generate spec.md from requirements
3. `/create-tasks` - Break down spec into tasks.md

**Key Features**:
- Separate phases for shaping vs writing
- Standards integration (or Claude Code Skills)
- Two-stage task creation (initialize → shape)
- Stores in `agent-os/specs/spec-{nn}/`
- Files: `planning/requirements.md`, `spec.md`, `planning/visuals/`, `tasks.md`

#### BERT
**Commands**: `/bert:spec new`, `/bert:spec iterate`, `/bert:spec tasks`

**Workflow**:
1. `/bert:spec new <desc>` - Create requirements.md with Q&A
2. `/bert:spec iterate {nn}` - **Smart iteration** (adds follow-ups OR generates spec OR updates spec)
3. `/bert:spec tasks {nn}` - Propose and create task files

**Key Features**:
- **Single iterate command** handles all refinement scenarios
- Auto-detects current state (requirements → spec → feedback)
- File-based async Q&A workflow
- Stores in `docs/bert/specs/spec-{nn}/`
- Files: `requirements.md`, `spec.md`, `visuals/`
- Creates separate task files: `task-{nn}.{sub}-{slug}.md`

**Winner**: **BERT** for simplicity (1 iterate vs 3 commands), **Agent-OS** for granular control

---

### 3. Task Management

#### Agent-OS
**No dedicated task management system**

- Tasks created from specs only
- Stored as single `tasks.md` file in spec directory
- No task-specific commands
- No hierarchical numbering
- Implementation via `/implement-tasks` or `/orchestrate-tasks`

#### BERT
**Command**: `/bert:task` (with subcommands)

**Features**:
- **Hierarchical tasks** - Unlimited nesting (3.1.2.4)
- **Ad-hoc tasks** - Create without specs (`/bert:task create "description"`)
- **Parent-child relationships** - `/bert:task create -p 3 "subtask"`
- **Status management** - `/bert:task status {nn} {status}`
- **Task listing** - `/bert:task list [filter]`
- **Archiving** - `/bert:task archive {nn}`
- **Spec-linked tasks** - Numbered as {spec}.{sub} (e.g., task-12.1, 12.2)
- Individual task files with detailed descriptions and acceptance criteria

**Winner**: **BERT** - Has dedicated task management; Agent-OS has no equivalent

---

### 4. Implementation & Execution

#### Agent-OS
**Commands**: `/implement-tasks` (simple), `/orchestrate-tasks` (advanced)

**`/implement-tasks` (single-agent)**:
1. Determine which tasks to implement
2. Implement tasks sequentially
3. Verify implementation (build, tests)
4. Create verification report

**`/orchestrate-tasks` (multi-agent)**:
1. Create `orchestration.yml` in spec directory
2. Assign Claude Code subagents to task groups
3. Assign standards to task groups (if not using Skills)
4. Delegate to subagents OR generate prompt files
5. Track progress in spec's `tasks.md`

**Key Features**:
- Standards compliance via Skills or file references
- Verification workflows built-in
- Multi-agent orchestration for complex features
- Generates detailed verification reports
- Updates roadmap after completion

#### BERT
**Command**: `/bert:task execute {nn}[,{nn},...]`

**Workflow**:
1. Read task file(s)
2. Implement task(s)
3. **Auto-generate review file** (`task-{nn}-review.md`)
4. User tests and adds issues to review file
5. User notifies AI: "added notes to task-{nn}-review.md"
6. AI reads issues, fixes them, documents fixes in review file
7. Repeat until "Ready for production" is checked

**Key Features**:
- **Auto-generated review files** - Single source of truth per feature
- **Async issue tracking** - Add issues anytime, AI fixes on notification
- **Execution can handle multiple tasks** - `/bert:task execute 1.2,1.3,1.4`
- **One review file per task hierarchy** - `task-01-review.md` covers all `01.x` tasks
- **Built-in testing checklist** in review file
- **Documented fix history** - Each issue shows status and fix details

**Winner**: **BERT** for developer workflow (review cycles), **Agent-OS** for large team orchestration

---

### 5. Review & Verification

#### Agent-OS
**Phase**: Part of `/implement-tasks` (Phase 3)

**Workflow**:
1. Run full test suite
2. Verify all tasks completed
3. Create verification report in `agent-os/specs/{spec}/implementation/verification/`
4. Update roadmap with completion
5. Optional: spec-verifier subagent

**Key Features**:
- Structured verification process
- Automated roadmap updates
- Test suite enforcement
- Formal verification reports

#### BERT
**Built into task execution** (automatic)

**Workflow**:
1. AI auto-creates review file after task execution
2. Review file contains:
   - Implementation summary
   - Files changed
   - Implementation notes
   - Testing checklist
   - Issues section (empty, for user to fill)
   - Final status checkboxes
3. User adds issues in Markdown format
4. AI fixes and documents each fix
5. Iterates until "Ready for production"

**Key Features**:
- **Review file is living document** - Updated throughout development
- **Async collaboration** - Add issues whenever convenient
- **Issue tracking built-in** - No external tools needed
- **Fix documentation automatic** - AI logs what it changed
- **Status-driven** - Clear "Ready for production" checkpoint

**Winner**: **BERT** - Review workflow is more developer-friendly and iterative

---

### 6. Configuration & Customization

#### Agent-OS 2.1
**Config File**: `config.yml` in base installation

**Configuration Options**:
```yaml
version: 2.1.0
claude_code_commands: true/false
agent_os_commands: true/false
use_claude_code_subagents: true/false
standards_as_claude_code_skills: true/false
profile: default
```

**Profiles System**:
- `profiles/default/` or custom profiles
- Contains: `agents/`, `commands/`, `standards/`, `workflows/`
- Switch profiles per project
- Custom project types with different standards

**Standards System**:
- Organized by category: `global/`, `backend/`, `frontend/`, `testing/`
- Can be Claude Code Skills or file references
- Configurable per task group in orchestration

**Installation**:
- Base installation + project installations
- `project-install.sh` with upgrade detection
- Copies files from base to project
- Self-contained per project

#### BERT 0.1.2
**Config File**: `.claude/skills/bert/skill.yml`

**Configuration Options**:
```yaml
config:
  tasks_directory: docs/bert/tasks
  notes_directory: docs/bert/notes
  specs_directory: docs/bert/specs
  product_directory: docs/bert/product
  # ... archive paths
```

**No Profiles System**:
- Single configuration per installation
- Simpler directory structure
- All paths customizable

**No Standards System**:
- Relies on Claude Code's general knowledge
- Can reference product context (mission, tech-stack)
- Less opinionated about coding standards

**Installation**:
- Single script: `base-install.sh`
- Installs directly to `.claude/` and `docs/bert/`
- No base/project separation

**Winner**: **Agent-OS** - Much more configurable and flexible

---

### 7. Directory Structure

#### Agent-OS 2.1
```
your-project/
├── agent-os/
│   ├── config.yml
│   ├── product/
│   │   ├── mission.md
│   │   ├── roadmap.md
│   │   └── tech-stack.md
│   └── specs/
│       └── spec-{nn}/
│           ├── planning/
│           │   ├── requirements.md
│           │   └── visuals/
│           ├── spec.md
│           ├── tasks.md
│           ├── orchestration.yml (if using orchestrate)
│           └── implementation/
│               ├── prompts/ (if not using subagents)
│               └── verification/
└── .claude/
    ├── commands/agent-os/
    ├── agents/agent-os/
    └── skills/ (if using standards_as_claude_code_skills)
```

#### BERT 0.1.2
```
your-project/
├── docs/bert/
│   ├── product/
│   │   ├── mission.md
│   │   ├── roadmap.md
│   │   └── tech-stack.md
│   ├── specs/
│   │   └── spec-{nn}/
│   │       ├── requirements.md
│   │       ├── spec.md
│   │       └── visuals/
│   ├── tasks/
│   │   ├── task-{nn}.md (ad-hoc)
│   │   ├── task-{nn}.{sub}-{slug}.md (from spec)
│   │   └── task-{nn}-review.md (auto-generated)
│   ├── notes/
│   └── archive/
│       ├── specs/
│       ├── tasks/
│       └── notes/
└── .claude/
    ├── commands/bert/
    └── skills/bert/
        ├── agents/
        ├── skill.md
        └── skill.yml
```

**Winner**: **BERT** - Cleaner separation of concerns (docs/ for content, .claude/ for tooling)

---

### 8. Agent System

#### Agent-OS 2.1
**Available Agents**:
- `product-planner.md`
- `spec-initializer.md`
- `spec-shaper.md`
- `spec-writer.md`
- `spec-verifier.md`
- `tasks-list-creator.md`
- `implementer.md`
- `implementation-verifier.md`

**Total**: 8 specialized agents

**Usage**: Phase-specific, delegated based on command

#### BERT 0.1.2
**Available Agents**:
- `requirements-gatherer.md`
- `spec-iterator.md` (handles all spec refinement)
- `task-proposer.md`
- `task-decomposer.md`

**Total**: 4 agents (but more consolidated)

**Usage**: Command-specific, spec-iterator is multi-purpose

**Winner**: **Agent-OS** - More specialized agents; **BERT** - Fewer but more versatile

---

### 9. Documentation

#### Agent-OS
- Comprehensive online docs at buildermethods.com/agent-os
- README with installation links
- CHANGELOG with detailed version history
- Separate upgrade guides
- Community support via Builder Methods Pro

#### BERT
- README with quick start and detailed workflows
- `docs/walkthrough.md` - Complete step-by-step example
- Inline command documentation
- Changelog in git history
- Credits Agent-OS as inspiration

**Winner**: **Agent-OS** - Professional documentation site; **BERT** - Better inline walkthrough

---

## What BERT Does Better

1. **Developer-Centric Review Workflow**
   - Auto-generated review files after every task execution
   - Built-in async issue tracking
   - AI fixes and documents issues on notification
   - Clear "Ready for production" checkpoints

2. **Simplified Command Structure**
   - 3 namespaced commands vs 6 phases
   - Single `/bert:spec iterate` vs multiple spec commands
   - More intuitive for beginners

3. **Hierarchical Task Management**
   - Dedicated task CRUD operations
   - Parent-child relationships
   - Ad-hoc tasks without specs
   - Task filtering and status management

4. **Cleaner Directory Structure**
   - `docs/bert/` for all user content
   - Separate task files (not single tasks.md)
   - Clear archive system

5. **File-Based Async Collaboration**
   - Edit requirements/specs on your time
   - Add issues to review files anytime
   - No need for immediate back-and-forth

6. **Smart Iteration**
   - One command auto-detects: requirements → spec → feedback
   - Less cognitive load on which command to run

---

## What Agent-OS Does Better

1. **Configuration Flexibility**
   - Profiles for different project types
   - Toggle subagents on/off
   - Choose Claude Code Skills vs file references
   - Customizable standards per task group

2. **Standards System**
   - Comprehensive coding standards
   - Organized by category (global, backend, frontend, testing)
   - Can be enforced via Skills or file references
   - Project-specific customization

3. **Multi-Agent Orchestration**
   - Advanced `/orchestrate-tasks` for complex features
   - Assign specialized subagents to task groups
   - Fine-grained control over agent contexts
   - Prompt file generation for non-Claude Code tools

4. **Professional Installation System**
   - Base + project installation separation
   - Upgrade detection and confirmation
   - Version tracking in config
   - Easy updates across projects

5. **Verification & Reporting**
   - Structured verification reports
   - Automated roadmap updates
   - Test suite enforcement
   - Formal completion tracking

6. **Granular Phase Control**
   - 6 distinct phases you can pick and choose
   - Clear separation: shape → write → create → implement → orchestrate
   - Better for large teams with defined roles

7. **Documentation & Community**
   - Professional documentation website
   - Active community via Builder Methods Pro
   - Regular updates and changelogs
   - Creator support

---

## When to Use Each

### Use Agent-OS 2.1 When:
- You need **coding standards enforcement** across a team
- You're building **complex features** requiring multi-agent orchestration
- You want **maximum configuration flexibility**
- You need to support **multiple project types** with different standards
- You prefer **phase-based workflows** with clear separation
- You want **professional support** and community
- Your team follows **strict spec-driven development**

### Use BERT 0.1.2 When:
- You want **fast, iterative development** with built-in review cycles
- You need **hierarchical task management** out of the box
- You prefer **simpler workflows** (3 commands vs 6 phases)
- You want **async issue tracking** without external tools
- You're a **solo developer** or small team
- You want to **mix ad-hoc tasks and spec-driven features**
- You value **developer ergonomics** over configuration options
- You prefer **living documentation** (review files) over formal reports

---

## Migration Considerations

### From Agent-OS to BERT
**Pros**:
- Simpler command structure
- Built-in review workflow
- Better task management

**Cons**:
- Lose standards system
- Lose multi-agent orchestration
- Lose profiles and configuration flexibility
- Smaller community

**Migration Effort**: **Medium** - Need to adapt to new command structure and directory layout

### From BERT to Agent-OS
**Pros**:
- Gain standards enforcement
- Gain advanced orchestration
- Gain professional support
- More configuration options

**Cons**:
- Lose review workflow (would need to build manually)
- Lose task hierarchy system
- Steeper learning curve
- More complex setup

**Migration Effort**: **High** - Significant architectural differences in workflow

---

## Conclusion

**BERT** and **Agent-OS** serve different needs despite sharing common ancestry:

- **Agent-OS 2.1** is a **comprehensive spec-driven development system** optimized for teams, standards compliance, and complex projects with extensive configuration options.

- **BERT 0.1.2** is a **developer-centric task execution tool** optimized for iterative development, built-in review cycles, and ease of use.

Neither is objectively "better" - they excel in different contexts. Agent-OS provides more power and flexibility at the cost of complexity. BERT provides speed and simplicity at the cost of advanced features.

**Best Hybrid Approach**: Use BERT's review workflow concept in Agent-OS, or use Agent-OS's standards system in BERT (see upgrade.md for recommendations).
