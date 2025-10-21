# Agent-OS Integration Recommendations

**Date**: 2025-10-21
**Status**: Proposal for Discussion

---

## Executive Summary

BERT is a developer-friendly task management tool that could benefit greatly from integrating with Agent-OS. During our analysis, we identified architectural improvements that would make Agent-OS easier to integrate with, extend, and build upon - not just for BERT, but for any tool in the ecosystem.

**Key recommendations**:
1. **Make paths configurable** (not hardcoded)
2. **Provide a clear API layer** (workflows as reusable modules)
3. **Support path overrides** in commands
4. **Standardize file reference patterns**
5. **Enable partial adoption** (use only what you need)

These changes would position Agent-OS as a **platform** that other tools can build on, rather than just a standalone system.

---

## Current Integration Challenges

### 1. Hardcoded Paths

Workflows hardcode `agent-os/specs/` and `agent-os/product/` paths, preventing tools from using custom directory structures.

**Example**: `SPEC_PATH="agent-os/specs/$DATED_SPEC_NAME"` (hardcoded)

→ _See [Technical Details: Path Configuration](agent-os-integration-technical.md#path-configuration-implementation) for implementation_

### 2. No Clear API Boundary

Workflows exist but aren't documented as reusable APIs, making it unclear what external tools can safely depend on.

→ _See [Technical Details: Workflows API Specification](agent-os-integration-technical.md#workflows-api-specification) for full API docs_

### 3. All-or-Nothing Adoption

Agent-OS feels like you must use everything, creating a higher barrier to entry.

→ _See [Technical Details: Example Integrations](agent-os-integration-technical.md#example-integrations) for modular usage patterns_

---

## Recommendations Overview

### 1. Make Paths Configurable

**Priority**: HIGH
**Effort**: Medium (2-3 weeks)
**Breaking Changes**: None

Add configuration system for paths:

```yaml
# config.yml
paths:
  specs_directory: "agent-os/specs"
  product_directory: "agent-os/product"
  specs_naming: "{date}-{name}"
```

Workflows read from environment variables with fallbacks:

```bash
SPECS_DIR="${SPECS_DIRECTORY:-agent-os/specs}"
SPEC_PATH="$SPECS_DIR/$DATED_SPEC_NAME"
```

**Benefits**:
- BERT can use `docs/bert/specs/`
- Other tools can customize paths
- Defaults match current behavior (backward compatible)

---

### 2. Document Workflows as Public APIs

**Priority**: HIGH
**Effort**: Low (1 week, documentation only)
**Breaking Changes**: None

Create "Workflows API" documentation for stable, reusable workflows:

```markdown
## workflows/specification/initialize-spec

**Stability**: Stable API v1.0
**Purpose**: Create a new spec folder structure

**Inputs** (environment variables):
- SPECS_DIRECTORY: Where to create spec (default: agent-os/specs)
- SPEC_NAME: Kebab-case spec name

**Outputs**:
- Creates folder structure
- Returns path to created spec

**Example**:
```bash
export SPECS_DIRECTORY="docs/myapp/specs"
export SPEC_NAME="authentication"
{{workflows/specification/initialize-spec}}
```
```

**Benefits**:
- Clear contracts for external tools
- Version stability guarantees
- Easier to build on Agent-OS
- Better testing boundaries

---

### 3. Support Path Overrides in Commands

**Priority**: MEDIUM
**Effort**: Medium (1-2 weeks)
**Breaking Changes**: None

Commands accept optional path flags:

```bash
/shape-spec "authentication" --specs-dir docs/myapp/specs
```

**Benefits**:
- Per-command flexibility
- No global config changes needed
- Backward compatible (flags optional)

---

### 4. Standardize File Reference Patterns

**Priority**: LOW
**Effort**: Low (documentation)
**Breaking Changes**: None

Document what each reference pattern means:

- `{{workflows/category/name}}` - Workflow reference
- `{{PHASE N: @agent-os/...}}` - Phase embedding
- `$VARIABLE` - Runtime path variable
- `{{IF flag}} ... {{ENDIF}}` - Conditional compilation

**Benefits**:
- Clarity for developers
- Easier to maintain
- Clear documentation

---

### 5. Enable Partial Adoption

**Priority**: MEDIUM
**Effort**: Low (documentation)
**Breaking Changes**: None

Document modular usage patterns:

**Use Case 1: Just Spec Writing**
- Install only spec workflows
- Skip product planning, tasks, implementation

**Use Case 2: Just Implementation**
- Install only implementation workflows
- Create custom task system
- Call implementer when ready

**Use Case 3: Spec + Custom Tasks** (BERT's approach)
- Use Agent-OS for specs
- Build custom task management
- Call Agent-OS implementer for execution

**Use Case 4: Full Agent-OS**
- Use everything as designed

**Benefits**:
- Lower barrier to entry
- "Use only what you need" philosophy
- Encourages ecosystem growth

---

## Benefits to Agent-OS

### 1. Becomes a Platform

**Current**: Agent-OS is a standalone system
**Future**: Agent-OS is a platform others build on

**Example ecosystem**:
- BERT: Task management + review workflow (powered by Agent-OS)
- Spec-Only Tool: Just uses Agent-OS spec generation
- Custom Implementer: Agent-OS specs + custom execution
- Enterprise: Agent-OS + custom standards + workflows

### 2. More Adoption

**Lower barriers**:
- "I like specs but not task breakdown" → Use just specs!
- "I need custom paths" → Configure them!
- "I want to extend Agent-OS" → Here's the API!

**Result**: More users, contributors, ecosystem growth

### 3. Better Maintainability

**Clear boundaries**:
- Public workflows (stable, versioned)
- Internal implementation (refactor freely)
- Tests focus on API contracts

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

1. **Path config**: Add to config.yml (optional), defaults match current
2. **Workflow APIs**: Documentation only, no code changes initially
3. **Command overrides**: New flags are optional
4. **Modular usage**: Documentation of existing capabilities

**Result**: Zero breaking changes, only additions

---

## Potential Collaboration

### What BERT Can Contribute

1. **First integration example**: BERT as reference implementation
2. **Testing**: Extensive testing of configurable paths
3. **Documentation**: Help document workflows API
4. **Use cases**: Real-world feedback
5. **Code contributions**: PRs for path configuration

### What BERT Needs from Agent-OS

1. **Path configuration system**: Core requirement
2. **Stable workflow APIs**: Won't break with updates
3. **Documentation**: Clear contracts
4. **Communication**: Heads up on breaking changes

---

## Next Steps

### For Discussion

1. **Path configuration**: Does the proposed approach align with your vision?
2. **Workflows API**: Would you consider stable, versioned workflow APIs?
3. **Integration examples**: Would you welcome BERT as official example?
4. **Roadmap alignment**: Do these fit with Agent-OS v2.x plans?
5. **Collaboration**: Interest in working together?

### Resources

- **Implementation details**: See [`agent-os-integration-technical.md`](agent-os-integration-technical.md)
  - Specific code changes and examples
  - Workflows API specification
  - Implementation roadmap with timelines
  - Testing and migration guides
- **BERT architecture**: See [`thin-bert-architecture-corrected.md`](thin-bert-architecture-corrected.md)

---

## Contact

For discussion:
- GitHub: https://github.com/buildermethods/agent-os/discussions/233
- Email: ssriram@gmail.com
- BERT discussions: https://github.com/ssr1ram/bert/discussions

---

## Conclusion

Agent-OS has incredible potential as a **platform** for spec-driven development tools. With relatively minor changes - primarily making paths configurable and documenting workflows as APIs - Agent-OS could become the foundation that many tools build upon.

**The vision**:
- Agent-OS = The powerful engine
- BERT = The customized UI
- Other tools = Custom integrations

**All powered by Agent-OS workflows.**

These changes benefit not just BERT, but the entire ecosystem. They position Agent-OS as the **standard platform** for AI-powered development workflows.

---

**Document Version**: 1.0
**See Also**:
- Technical implementation details: `agent-os-integration-technical.md`
- BERT + Agent-OS architecture: `thin-bert-architecture-corrected.md`
