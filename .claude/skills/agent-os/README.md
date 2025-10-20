# Agent-OS Skill

> Multi-agent specification and implementation system for structured product development

## Overview

Agent-OS is a comprehensive framework for spec-driven development that orchestrates multiple specialized agents to handle planning, implementation, and verification of complex features. It provides structured workflows for product planning, specification creation, and multi-phase implementation with built-in quality assurance.

## Features

- **Spec-Driven Development**: Create comprehensive specifications with automated task breakdown
- **Multi-Agent Orchestration**: Delegate implementation and verification to specialized subagents
- **Product Management**: Mission statements, roadmaps, and tech stack documentation
- **Quality Assurance**: Built-in verification workflows with implementer/verifier role separation
- **Standards Enforcement**: Global, backend, frontend, and testing standards

## Installation

### Prerequisites

- Claude Code (2025 or later)
- Skills support (available on Max, Pro, Teams, and Enterprise plans)

### Method 1: Manual Installation

1. Copy the entire `.claude/skills/agent-os/` directory to your project's `.claude/skills/` directory
2. Copy the `agent-os/` data directory to your project root
3. The skill will be automatically available in Claude Code

```bash
# In your project directory
cp -r /path/to/source/.claude/skills/agent-os .claude/skills/
cp -r /path/to/source/agent-os ./
```

### Method 2: Using Installation Script

```bash
# Run the installation script (if provided)
bash install-agent-os.sh
```

### Method 3: Git Submodule (For Teams)

```bash
# Add as a git submodule for version control
git submodule add <repository-url> .claude/skills/agent-os
git submodule update --init --recursive

# Team members can then get it via:
git submodule update --init --recursive
```

## Directory Structure

After installation, you'll have:

```
.claude/skills/agent-os/
├── README.md           # This file
├── skill.yml           # Skill configuration
├── skill.md            # Skill instructions
└── agents/             # 19 specialized agent definitions
    ├── spec-initializer.md
    ├── spec-researcher.md
    ├── spec-writer.md
    ├── tasks-list-creator.md
    ├── spec-verifier.md
    ├── product-planner.md
    ├── implementation-verifier.md
    └── [14 more agents...]

agent-os/               # Data directory (project root)
├── product/            # Product documentation
│   ├── mission.md
│   ├── roadmap.md
│   └── tech-stack.md
├── roles/              # Role definitions
│   ├── implementers.yml
│   └── verifiers.yml
├── standards/          # Coding standards
│   ├── global/
│   ├── backend/
│   ├── frontend/
│   └── testing/
└── specs/              # Active specifications
    └── [spec directories...]
```

## Usage

Once installed, use agent-os with natural language commands:

### Product Planning

```
"Plan the product"
"Create product documentation"
```

Creates mission, roadmap, and tech stack documentation.

### Specification Creation

```
"Create a new spec for user authentication"
"Initialize spec for payment processing"
```

Initializes spec folder and gathers requirements through interactive Q&A.

```
"Create the spec"
"Generate the full specification"
```

Generates comprehensive specification and task breakdown.

### Implementation

```
"Implement the spec"
"Start implementing the authentication spec"
```

Executes multi-phase implementation with specialized agents and verification.

## Operations

Agent-OS provides 4 main operations:

1. **new-spec** - Initialize new specification with requirements gathering
2. **create-spec** - Generate comprehensive specification and task breakdown
3. **implement-spec** - Multi-phase implementation with delegated agents
4. **plan-product** - Create product mission, roadmap, and tech stack

## Agents

Agent-OS includes 19 specialized agents:

**Spec Creation**:
- spec-initializer, spec-researcher, spec-writer, tasks-list-creator, spec-verifier

**Product Planning**:
- product-planner

**Implementation**:
- system-architect, agent-storage-dev, agent-orchestrator-dev, agent-research-dev
- agent-factcheck-dev, integration-engineer, testing-engineer, database-engineer
- api-engineer, ui-designer

**Verification**:
- implementation-verifier, backend-verifier, frontend-verifier

## Configuration

Edit `.claude/skills/agent-os/skill.yml` to customize:

- `specs_directory` - Where specifications are stored (default: `agent-os/specs`)
- `product_directory` - Product documentation location (default: `agent-os/product`)
- `standards_directory` - Coding standards location (default: `agent-os/standards`)

## Workflow Example

```bash
# 1. Plan your product
"Plan the product for a news aggregation system"

# 2. Create a new spec
"Create a new spec for article collection and scoring"
# Answer the requirements questions

# 3. Generate the specification
"Create the spec"

# 4. Implement the specification
"Implement the spec"
# Agent-OS will delegate to specialized agents and verify
```

## Customization

### Adding Custom Agents

1. Create a new agent file in `.claude/skills/agent-os/agents/`
2. Add the agent role to `agent-os/roles/implementers.yml` or `verifiers.yml`
3. Reference the agent in your workflows

### Adding Custom Standards

1. Add standard files to `agent-os/standards/[category]/`
2. Reference standards in your specifications
3. Agents will automatically enforce these standards during implementation

## Sharing with Your Team

### Via Git Repository

```bash
# Include in your project repository
git add .claude/skills/agent-os agent-os/
git commit -m "Add agent-os skill"
git push

# Team members get it automatically when they clone
```

### Via Package/Archive

```bash
# Create a distributable package
tar -czf agent-os-skill.tar.gz .claude/skills/agent-os agent-os/

# Share the archive
# Others can extract it in their project:
tar -xzf agent-os-skill.tar.gz
```

### Via Shared Location

```bash
# Create installation script pointing to shared location
# See install-agent-os.sh for example
```

## Troubleshooting

**Skill not appearing**:
- Ensure `.claude/skills/agent-os/` exists in your project
- Check that `skill.yml` and `skill.md` are present
- Restart Claude Code if needed

**Agents not found**:
- Verify all 19 agent files are in `.claude/skills/agent-os/agents/`
- Check `agent-os/roles/` contains implementers.yml and verifiers.yml

**Directory errors**:
- Ensure `agent-os/` directory exists at project root
- Create missing directories: `product/`, `roles/`, `standards/`, `specs/`

## Version

Current version: 2.0.3

## License

ISC

## Support

For issues, questions, or contributions, please contact the skill maintainer or open an issue in your project repository.

## Credits

Agent-OS was developed as part of the wodo-news project for structured, multi-agent development workflows.
