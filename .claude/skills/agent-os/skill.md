# Agent-OS - Spec-Driven Development System

You are now activating the **Agent-OS** skill for multi-agent specification and implementation.

## Overview

Agent-OS is a comprehensive framework for spec-driven development that orchestrates multiple specialized agents to handle planning, implementation, and verification of complex features. It provides structured workflows for product planning, specification creation, and multi-phase implementation with built-in quality assurance.

## Configuration

**Read the config file** `.claude/skills/agent-os/skill.yml` to determine directory locations for all operations.

Key directories:
- **Specs**: `agent-os/specs/` - Active specifications and implementations
- **Product**: `agent-os/product/` - Mission, roadmap, tech stack
- **Standards**: `agent-os/standards/` - Coding conventions (global, backend, frontend, testing)
- **Roles**: `agent-os/roles/` - Implementer and verifier role definitions
- **Agents**: `.claude/skills/agent-os/agents/` - Agent prompt definitions

## Core Operations

### 1. Plan Product (`plan-product`)

Create or update product documentation including mission, roadmap, and tech stack.

**Usage Pattern**: User says "plan the product" or "create product documentation"

**Workflow**:

#### PHASE 1: Gather Product Requirements

Use the **product-planner** subagent (located at `.claude/skills/agent-os/agents/product-planner.md`):

```
Provide to product-planner:
- Any details the user has provided about:
  - Product idea and purpose
  - Features list
  - Target users
  - Tech stack preferences
```

The product-planner will:
- Confirm or gather product vision, features, target users
- Confirm the tech stack
- Create `agent-os/product/mission.md` - Product vision and strategy
- Create `agent-os/product/roadmap.md` - Phased development plan
- Create `agent-os/product/tech-stack.md` - Technical stack documentation

#### PHASE 2: Display Results

Show the user:
- Confirmation of files created
- Summary of product mission
- Overview of roadmap phases

### 2. New Spec (`new-spec`)

Initialize a new specification with requirements gathering.

**Usage Pattern**: User says "create a new spec for [feature]" or "initialize spec"

**Workflow**:

#### PHASE 1: Initialize Spec

Use the **spec-initializer** subagent (`.claude/skills/agent-os/agents/spec-initializer.md`):

```
Provide to spec-initializer:
- Description of the feature (if user provided one)
```

The spec-initializer will:
- Create a dated spec folder: `agent-os/specs/YYYY-MM-DD-feature-name/`
- Set up directory structure: `planning/`, `implementation/`, `verification/`
- Return the spec folder path

#### PHASE 2: Research Requirements

Use the **spec-researcher** subagent (`.claude/skills/agent-os/agents/spec-researcher.md`):

```
Provide to spec-researcher:
- The spec folder path from spec-initializer
```

The spec-researcher will:
- Ask clarifying questions (DISPLAY THESE TO USER)
- Request visual assets if applicable
- May ask follow-up questions based on responses
- Create `planning/requirements.md` with gathered requirements

**IMPORTANT**: Display all questions to the user and wait for their responses before proceeding.

#### PHASE 3: Inform User

After completion, tell the user:

```
Spec initialized successfully!

✅ Spec folder created: `[spec-path]`
✅ Requirements gathered
✅ Visual assets: [Found X files / No files provided]

👉 Next step: Use agent-os skill to run 'create-spec' to generate the detailed specification and task breakdown.
```

### 3. Create Spec (`create-spec`)

Generate comprehensive specification and task breakdown from gathered requirements.

**Usage Pattern**: User says "create the spec" after running new-spec

**Workflow**:

#### PHASE 1: Delegate to Spec Writer

Use the **spec-writer** subagent (`.claude/skills/agent-os/agents/spec-writer.md`):

```
Provide to spec-writer:
- The spec folder path (find most recent in `agent-os/specs/*/`)
- The requirements from `planning/requirements.md`
- Any visual assets in `planning/visuals/`
```

The spec-writer will:
- Create comprehensive `spec.md` in the spec folder
- Include architecture, data models, workflows, edge cases

**Wait for spec-writer to complete before proceeding to PHASE 2.**

#### PHASE 2: Delegate to Tasks List Creator

Use the **tasks-list-creator** subagent (`.claude/skills/agent-os/agents/tasks-list-creator.md`):

```
Provide to tasks-list-creator:
- The spec folder path
- The `spec.md` file that was just created
- The original requirements from `planning/requirements.md`
- Any visual assets in `planning/visuals/`
```

The tasks-list-creator will:
- Break down spec into task groups
- Create strategic ordering and grouping
- Generate `tasks.md` in the spec folder

#### PHASE 3: Verify Specifications

Use the **spec-verifier** subagent (`.claude/skills/agent-os/agents/spec-verifier.md`):

```
Provide to spec-verifier:
- ALL questions asked to user during requirements gathering
- ALL user's raw responses to those questions
- The spec folder path
```

The spec-verifier will:
- Verify spec accuracy against requirements
- Produce verification report in `verification/spec-verification.md`

#### PHASE 4: Display Results

Show the user:
- Spec creation summary from spec-writer
- Tasks list creation summary from tasks-list-creator
- Verification summary from spec-verifier
- Highlight any issues found

Expected output structure:
```
agent-os/specs/[date-spec-name]/
├── planning/
│   ├── initialization.md
│   ├── requirements.md
│   └── visuals/
├── verification/
│   └── spec-verification.md
├── spec.md
└── tasks.md
```

### 4. Implement Spec (`implement-spec`)

Execute multi-phase implementation with delegated agents and verification.

**Usage Pattern**: User says "implement the spec" after create-spec is complete

**Workflow**:

#### PHASE 1: Plan Subagent Assignments

1. Read `agent-os/specs/[this-spec]/tasks.md`
2. Read `agent-os/roles/implementers.yml`
3. Create `agent-os/specs/[this-spec]/planning/task-assignments.yml`:

```yaml
task_assignments:
  - task_group: "Task Group 1: [Title from tasks.md]"
    assigned_subagent: "[implementer-id-from-implementers.yml]"

  - task_group: "Task Group 2: [Title from tasks.md]"
    assigned_subagent: "[implementer-id-from-implementers.yml]"
```

Verify each assigned subagent exists in:
- `agent-os/roles/implementers.yml` (role definition)
- `.claude/skills/agent-os/agents/[subagent-id].md` (agent prompt)

#### PHASE 2: Delegate Task Groups to Implementers

For each task group in `tasks.md`:

1. Find the assigned subagent from `task-assignments.yml`
2. Delegate to that subagent:

```
Provide to implementer:
- The task group (parent task + all sub-tasks)
- The spec file: `agent-os/specs/[this-spec]/spec.md`

Instruct implementer to:
1. Perform the implementation
2. Check off tasks in `agent-os/specs/[this-spec]/tasks.md`
3. Document work in `agent-os/specs/[this-spec]/implementation/[task-name].md`
```

#### PHASE 3: Delegate to Verifier Subagents

1. Collect list of subagent IDs used in Phase 2
2. Read `agent-os/roles/implementers.yml` and find their `verified_by` fields
3. Read `agent-os/roles/verifiers.yml` for verifier definitions
4. For each verifier role, delegate verification:

```
Provide to verifier:
- All task groups under this verifier's purview
- The spec file: `agent-os/specs/[this-spec]/spec.md`

Instruct verifier to:
1. Analyze tasks in context of spec
2. Run tests to verify implementation
3. Verify `tasks.md` is updated correctly
4. Document verification in `agent-os/specs/[this-spec]/verification/[verifier-name].md`
```

#### PHASE 4: Final Verification

Use the **implementation-verifier** subagent (`.claude/skills/agent-os/agents/implementation-verifier.md`):

```
Provide to implementation-verifier:
- The spec path: `agent-os/specs/[this-spec]`

Instruct to:
1. Run all final verifications per built-in workflow
2. Produce final report in `agent-os/specs/[this-spec]/verification/final-verification.md`
```

## Agent Registry

All agents are stored in `.claude/skills/agent-os/agents/` directory.

**Spec Creation Agents**:
- `spec-initializer.md` - Initialize new spec folders
- `spec-researcher.md` - Gather requirements through user Q&A
- `spec-writer.md` - Write comprehensive specifications
- `tasks-list-creator.md` - Break down specs into task lists
- `spec-verifier.md` - Verify specs against requirements

**Product Planning Agents**:
- `product-planner.md` - Create product documentation

**Implementation Agents** (defined in `agent-os/roles/implementers.yml`):
- `system-architect.md` - Project setup, config, schemas
- `agent-storage-dev.md` - File I/O, data persistence
- `agent-orchestrator-dev.md` - Command parsing, workflow coordination
- `agent-research-dev.md` - Web search, article discovery
- `agent-factcheck-dev.md` - Relevance scoring, verification
- `integration-engineer.md` - Agent communication, workflows
- `testing-engineer.md` - Test coverage, quality assurance
- `database-engineer.md` - Database schemas and queries
- `api-engineer.md` - API design and implementation
- `ui-designer.md` - UI/UX design

**Verification Agents** (defined in `agent-os/roles/verifiers.yml`):
- `implementation-verifier.md` - End-to-end system verification
- `backend-verifier.md` - Backend code verification
- `frontend-verifier.md` - Frontend code verification

## Standards

Agent-OS enforces coding standards stored in `agent-os/standards/`:

- `global/` - Standards applicable to all code
- `backend/` - Backend-specific standards
- `frontend/` - Frontend-specific standards
- `testing/` - Testing standards and practices

All implementer agents should reference relevant standards when implementing their tasks.

## Key Behaviors

1. **Always read config**: Determine directories from `.claude/skills/agent-os/skill.yml`
2. **Sequential phases**: Complete each phase before moving to next
3. **Agent delegation**: Use Task tool with appropriate subagent prompts
4. **State management**: All state stored in spec folders on disk
5. **Verification**: Every implementation includes verification phase
6. **Standards compliance**: Reference standards during implementation
7. **User confirmation**: Display questions and wait for responses

## User Interaction Patterns

When users say:
- "Plan the product" → Use plan-product operation
- "Create a new spec for X" → Use new-spec operation
- "Create the spec" → Use create-spec operation (after new-spec)
- "Implement the spec" → Use implement-spec operation (after create-spec)

## Session Context

Agent-OS is now **ACTIVE** for this session. Natural language commands will invoke the appropriate operations:
- "I want to plan out the product vision"
- "Let's create a spec for the authentication system"
- "Generate the full specification"
- "Start implementing the spec"

Ready to manage your product development with Agent-OS.
