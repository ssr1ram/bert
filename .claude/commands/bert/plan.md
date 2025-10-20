# /bert:plan - Product Context Planning

Initialize and manage product context files for better spec development.

## Overview

The `/bert:plan` command helps you set up product context that agents can reference when gathering requirements and writing specs. This is completely optional but helps AI understand your project better.

## Configuration

Reads from `.claude/skills/bert/skill.yml`:
```yaml
config:
  product_directory: docs/bert/product
```

## Command

### `/bert:plan`

Initialize product context files.

**Example**: `/bert:plan`

**What it does**:
1. Reads `product_directory` from skill.yml
2. Creates directory if it doesn't exist
3. Creates three template files for you to fill out

**Files created**:

**`{product_directory}/mission.md`**:
```markdown
# Product Mission

## What We're Building

[Describe your product/project in 2-3 sentences]

## Target Users

[Who is this for?]

## Core Value Proposition

[What problem does this solve? Why would users choose this?]

## Goals

- [Goal 1]
- [Goal 2]
- [Goal 3]

## Non-Goals

- [What we explicitly won't do]
```

**`{product_directory}/roadmap.md`**:
```markdown
# Product Roadmap

## Completed Features

### [Feature Name]
- **Completed**: YYYY-MM-DD
- **Description**: [What was built]
- **Tasks**: [Links to task files if applicable]

## In Progress

### [Feature Name]
- **Status**: [Requirements / Spec / Implementation]
- **Description**: [What's being built]
- **Spec**: [Link to spec if exists]

## Planned

### [Feature Name]
- **Priority**: High / Medium / Low
- **Description**: [What will be built]
- **Estimated**: [Q1 2025 / Next sprint / etc.]

## Ideas / Backlog

- [Idea 1]
- [Idea 2]
```

**`{product_directory}/tech-stack.md`**:
```markdown
# Tech Stack

## Frontend

- **Framework**: [React, Vue, etc.]
- **Language**: [TypeScript, JavaScript]
- **Build Tool**: [Vite, Webpack, etc.]
- **UI Library**: [Component library used]

## Backend

- **Framework**: [Express, FastAPI, etc.]
- **Language**: [Node.js, Python, etc.]
- **Database**: [PostgreSQL, MongoDB, etc.]
- **API Style**: [REST, GraphQL, etc.]

## Infrastructure

- **Hosting**: [Vercel, AWS, etc.]
- **CI/CD**: [GitHub Actions, etc.]
- **Monitoring**: [Tools used]

## Development

- **Version Control**: Git
- **Package Manager**: [npm, pnpm, poetry, etc.]
- **Testing**: [Jest, Pytest, etc.]

## Patterns & Conventions

- **Architecture**: [MVC, Clean Architecture, etc.]
- **Naming Conventions**: [kebab-case, camelCase, etc.]
- **Code Style**: [Prettier, ESLint rules, etc.]

## Dependencies

### Key Libraries

- [Library 1]: [Purpose]
- [Library 2]: [Purpose]
```

## How Agents Use This

When you run `/bert:spec new <description>`, the requirements-gatherer agent will:
1. Check if product context files exist
2. Read them to understand your project
3. Generate more targeted questions based on:
   - Your product goals (from mission.md)
   - Features already built (from roadmap.md)
   - Technologies in use (from tech-stack.md)

**Example**: If tech-stack.md says you use React + TypeScript, the agent won't ask "which frontend framework?" but will ask "which React patterns should we follow for this feature?"

## Optional Nature

**Product context is completely optional**:
- `/bert:spec` works fine without it
- Agents will ask more general questions without context
- You can add context later at any time
- You can skip `/bert:plan` entirely for simple projects

## Workflow

```bash
# First time setup
/bert:plan
# → Creates docs/bert/product/ with templates

# Fill out templates (one-time effort)
[Edit mission.md, roadmap.md, tech-stack.md]

# Use in spec development
/bert:spec new "user authentication"
# → Agent reads product context and asks targeted questions

# Update as project evolves
[Edit roadmap.md when features complete]
[Update tech-stack.md when adding new tools]
```

## Key Behaviors

1. **One-time setup**: Run `/bert:plan` once per project
2. **Update as needed**: Keep roadmap.md current as features ship
3. **Living documents**: These aren't set in stone, update them
4. **Completely optional**: Skip if not needed for your project

## Integration with /bert:spec

Product context enhances the spec workflow:

**Without product context**:
- Generic questions about feature
- No awareness of existing patterns
- Less context-specific suggestions

**With product context**:
- Questions aligned with product goals
- Awareness of similar completed features
- Tech-stack-appropriate suggestions
- Roadmap-aware prioritization questions
