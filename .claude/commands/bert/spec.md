# /bert:spec - Specification Development

Create and iterate on specifications for complex features with intelligent requirements gathering and task breakdown.

## Overview

The `/bert:spec` command provides a streamlined workflow for planning complex features:
1. Create requirements with Q&A
2. Iterate to refine requirements and generate specs
3. Create task files when ready

**Specs are numbered** (spec-12, spec-13) and their tasks use that number as prefix (task-12.1, 12.2, etc.)

## Configuration

Read paths from `.claude/skills/bert/skill.yml`:
```yaml
config:
  specs_directory: docs/bert/specs
  tasks_directory: docs/bert/tasks
  product_directory: docs/bert/product  # optional
```

## Commands

### `/bert:spec new <description>`

Start a new spec with requirements gathering.

**Example**: `/bert:spec new "user authentication system"`

**Workflow**:
1. **Determine next spec number** (check existing specs):
   - Scan `{specs_directory}/` for existing spec directories
   - Find highest spec number (e.g., spec-01 → 1)
   - Increment by 1 and pad to 2 digits: `printf "%02d" $((max + 1))`
   - Example: spec-01 exists → next is spec-02 (NOT spec-1 or spec-2)
2. Create spec directory: `{specs_directory}/spec-{nn}/` (2-digit padding required)
3. Invoke `requirements-gatherer` agent with spec number
4. Agent creates `requirements.md` with Q&A template
5. Agent checks for product context files (optional)
6. Agent generates 6-9 targeted questions
7. User fills out answers asynchronously

**Output**:
```
{specs_directory}/spec-{nn}/
  requirements.md              # Q&A template, status: awaiting-answers
  visuals/                     # Empty directory for mockups
```

**Next steps**:
- Fill in your answers in requirements.md
- Optionally add visual assets to visuals/
- Run: `/bert:spec iterate {nn}` when ready

---

### `/bert:spec iterate <spec_number>`

Process user feedback and generate/regenerate specification.

**Example**: `/bert:spec iterate 12`

**What it does**:
This is the **smart iteration command** that handles multiple scenarios:

**Scenario 1: First iteration (no spec exists)**
1. Read requirements.md
2. Extract your answers
3. Check for visual assets in visuals/
4. Determine if follow-up questions needed
5. **If follow-ups needed**: Add questions to requirements.md, STOP
6. **If requirements complete**: Generate spec.md from requirements

**Scenario 2: Iterating on requirements (spec doesn't exist)**
1. Read requirements.md with your latest answers
2. Process new answers
3. Add more follow-ups if needed, OR
4. Generate spec.md if requirements now complete

**Scenario 3: Iterating on spec (spec exists)**
1. Read spec.md
2. Extract your feedback from feedback section
3. Update spec based on feedback
4. Increment iteration number
5. Clear feedback section for next round

**The command is smart**: It detects the current state and does the right thing.

**Workflow examples**:

**Example A: Requirements need refinement**
```bash
/bert:spec new "authentication"
[Fill answers in requirements.md]
/bert:spec iterate 12
# → Reads answers, adds follow-up questions
[Answer follow-ups]
/bert:spec iterate 12
# → Reads follow-ups, generates spec.md
```

**Example B: Requirements complete first try**
```bash
/bert:spec new "authentication"
[Fill detailed answers in requirements.md]
/bert:spec iterate 12
# → Reads answers, generates spec.md immediately
```

**Example C: Spec needs refinement**
```bash
/bert:spec iterate 12
# → spec.md exists, read it first time
[Add feedback in spec.md feedback section]
/bert:spec iterate 12
# → Updates spec based on feedback
[Add more feedback if needed]
/bert:spec iterate 12
# → Updates again
```

**Key behavior**:
- Reads feedback from files (requirements.md or spec.md)
- No separate commands for requirements vs spec feedback
- One iterate command handles all scenarios
- User controls pace (can iterate multiple times)

---

### `/bert:spec tasks <spec_number>`

Create Bert task files from approved spec.

**Example**: `/bert:spec tasks 12`

**Prerequisites**: spec.md must exist (run `/bert:spec iterate` first)

**Workflow**:
1. Read `{specs_directory}/spec-{nn}/spec.md`
2. Invoke `task-proposer` agent
3. Agent analyzes spec and proposes task breakdown
4. Agent shows proposed tasks with dependencies
5. User approves or requests changes
6. Agent creates task files: `{tasks_directory}/task-{nn}.{sub}-{slug}.md`

**Output**:
```
{tasks_directory}/
  task-12.1-database-schema.md
  task-12.2-api-endpoints.md
  task-12.3-frontend-components.md
```

**Next steps**:
- Execute tasks with `/bert:task execute {nn}.{sub}`
- If you realize spec needs changes, you can:
  - Edit spec.md
  - Run `/bert:spec iterate {nn}` to regenerate
  - Run `/bert:spec tasks {nn}` again (overwrites previous tasks)

---

## Complete Workflow Example

```bash
# 1. Start new spec
/bert:spec new "user authentication system"
# → Creates docs/bert/specs/spec-12/requirements.md

# 2. Answer questions (async, multi-session)
[Edit docs/bert/specs/spec-12/requirements.md]
[Add mockups to docs/bert/specs/spec-12/visuals/ if available]

# 3. First iteration
/bert:spec iterate 12
# → Reads answers, may add follow-ups OR generate spec

# 4. If follow-ups added, answer and iterate again
[Answer follow-ups in requirements.md]
/bert:spec iterate 12
# → Now generates spec.md

# 5. Review spec, add feedback
[Read docs/bert/specs/spec-12/spec.md]
[Add feedback in feedback section]

# 6. Iterate on spec
/bert:spec iterate 12
# → Updates spec based on feedback

# 7. Repeat step 5-6 until satisfied
[Keep iterating until spec looks good]

# 8. Create tasks
/bert:spec tasks 12
# → Proposes task breakdown
[Review proposed tasks]
[Approve or modify]
# → Creates task-12.1.md, task-12.2.md, etc.

# 9. Realize spec incomplete after seeing tasks
[Edit spec.md to add missing details]
/bert:spec iterate 12
# → Regenerates spec
/bert:spec tasks 12
# → Regenerates tasks with updated spec

# 10. Execute tasks
/bert:task execute 12.1
/bert:task execute 12.2
```

## Spec Directory Structure

```
docs/bert/
├── specs/
│   └── spec-12/                    # Spec directory
│       ├── requirements.md         # Q&A with user
│       ├── spec.md                 # Technical specification
│       └── visuals/                # Optional mockups/wireframes
│           ├── login-mockup.png
│           └── auth-flow.pdf
└── tasks/
    ├── task-12.1-database-schema.md      # Created from spec
    ├── task-12.2-api-endpoints.md        # Created from spec
    └── task-12.3-frontend-components.md  # Created from spec
```

## Specs vs Tasks

**Specs** are numbered (spec-12, spec-13) and live in `docs/bert/specs/`:
- Each spec has its own directory: `docs/bert/specs/spec-{nn}/`
- Contains: requirements.md, spec.md, visuals/

**Tasks** created from specs use the spec number as prefix:
- Spec 12 → tasks 12.1, 12.2, 12.3, etc.
- Still live in `docs/bert/tasks/` as `task-12.1-{slug}.md`
- Link back to their parent spec

**Ad-hoc tasks** created without specs:
- `/bert:task create "fix login bug"` → task-15-fix-login-bug.md (no spec)

## Key Behaviors

1. **Read config**: Always get directories from skill.yml
2. **Numbered specs**: spec-12, spec-13, etc.
3. **Linked tasks**: Tasks from spec 12 are 12.1, 12.2, 12.3
4. **File-based feedback**: Write feedback in files, /bert:spec iterate reads it
5. **Smart iteration**: One command handles requirements + spec feedback
6. **No lock-in**: Can use /bert:spec or skip straight to /bert:task
7. **Product context optional**: Works with or without product files from /bert:plan
8. **Regeneration allowed**: Can iterate and regenerate tasks if spec changes

## Agents Used

- `requirements-gatherer`: `.claude/skills/bert/agents/requirements-gatherer.md`
- `spec-writer`: `.claude/skills/bert/agents/spec-writer.md`
- `task-proposer`: `.claude/skills/bert/agents/task-proposer.md`

All agents read config from skill.yml for paths.

## Integration with Other Commands

**With `/bert:plan`**:
```bash
/bert:plan                          # Setup product context (optional)
/bert:spec new "feature"            # Agent uses product context
```

**With `/bert:task`**:
```bash
/bert:spec tasks 12                 # Create task files
/bert:task execute 12.1             # Execute first task
/bert:task status 12.1 completed    # Mark complete
```

## Removed Commands

The following commands from the old `/spec` workflow are **removed**:

- ❌ `/spec process-requirements` - Merged into `/bert:spec iterate`
- ❌ `/spec write` - Merged into `/bert:spec iterate`
- ❌ `/spec process-feedback` - Merged into `/bert:spec iterate`
- ❌ `/spec approve` - Redundant, just run `/bert:spec tasks` when ready
- ❌ `/spec list` - Not needed, use `ls docs/bert/specs`
- ❌ `/spec show` - Not needed, use `cat` or open files
- ❌ `/spec init-product` - Moved to `/bert:plan`
- ❌ `/spec create-tasks` - Renamed to `/bert:spec tasks`

## Why This Is Better

**Old workflow** (7 commands, confusing):
```bash
/spec new → /spec process-requirements → /spec write →
/spec process-feedback → /spec approve → /spec create-tasks
```

**New workflow** (3 commands, clear):
```bash
/bert:spec new → /bert:spec iterate (repeat as needed) → /bert:spec tasks
```

**Benefits**:
- One iterate command instead of 4 different commands
- No confusion about which command to run
- Can iterate as many times as needed
- Smart detection of current state
- Simpler mental model
