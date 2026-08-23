# Changelog

All notable changes are documented here.

## [0.3.3] - 2026-08-23

### Fixed
- `spec archive` now uses the tolerant filename parser: finds bare-format parents (`task-013.md`) and grandchild recursion/notes lookup no longer break on bare-format children
- `cargo test` is green again end-to-end: uncompilable `tests/sets_test.rs` removed (round-trip tests moved into `models/set.rs`); three integration tests updated from pre-zero-config expectations

### Changed - Simplification pass (behavior-preserving)
- Shared test fixture replaces 12 copies of the `BertConfig` literal
- One directory-tree scanner shared by FileViewer and PromptBuilder; one expand-all walker; shared menu-item text between render and click hit-testing; `find_notes_file`/`find_task_file` shared across task/spec archive
- Recursive archive scans each directory once instead of rescanning per descendant
- `bert task adopt`, parent-stub numbering: single walk instead of two
- Removed dead code: `command_selector.rs` (never compiled), `anyhow`/`walkdir` deps, write-only state fields, never-constructed `YamlError`, vestigial TUI parameter plumbing
- TUI: prompt builder borrows config instead of cloning per frame/keypress; irrelevant mouse events drained instead of triggering redraws; input-mode editing without per-key enum clones
- TOCTOU guards removed where the follow-up call already handled absence; `prompt_stub` keeps propagating real read errors via error-kind match

## [0.3.2] - 2026-08-23

### Changed - Flag Renaming
- `--bert-dir` renamed to `--reporoot` (aliases: `--repo-root`, `--repodir`); overrides the repo root otherwise discovered via git
- `--tasks-dir` renamed to `--taskdir` (aliases: `--task-dir`, hidden `--tasks-dir`)
- No CLI option carries the "bert" name anymore

## [0.3.1] - 2026-08-23

### Changed - Self-Contained Zero-Config Layout
- With no `config:` section, bert's entire footprint is the tasks directory: archive, notes, specs, prompts and product context now nest inside it (`docs/tasks/archive`, `docs/tasks/notes`, ...) instead of claiming sibling `docs/` names
- An explicit `config:` section keeps the classic bert_root-derived layout unchanged

## [0.3.0] - 2026-08-23

### Added - Zero-Config Discovery & Format Tolerance

**Project Discovery**
- Repo root now found via git (`git rev-parse --show-toplevel`) instead of walking up for a config file
- Zero-config mode: tasks default to `<repo_root>/docs/tasks`; no `skills.yml` required
- Optional `.bert/config.yml` (dot-directory convention); legacy `skills.yml` still honored
- New global flag: `--tasks-dir`, alongside existing `--bert-dir`

**Format Tolerance**
- Lenient task-filename parser: bare `task-013.md`, slugged `task-01-x.md`, and dotted subtasks all read
- Numeric task-number matching (`bert task archive 7` finds `task-007.md`)
- Status synonym normalization: open/pending→todo, in-progress→doing, paused/deferred→parked, done/completed→done
- New stubs mimic the existing directory's filename shape, number padding, frontmatter keys, status word, and H1 style

**New Commands**
- `bert task adopt` — detect a tasks directory's conventions and persist them to `.bert/config.yml`
- `bert task list` — filterable listing (`--status/--track/--priority/--tag`), aligned table or `--json`

### Fixed
- Stale `ProjectNotFound` error message referencing `.claude/skills/bert/skill.yml`
- Binary now works in the bert repo itself (previously failed without a root `skills.yml`)

## [0.2.2] - 2025-10-26

### Added - Phase 1: Standards Adoption

**Standards System**
- Added `docs/bert/standards/` directory structure (compatible with Agent-OS 2.1)
- Created 15 standards files across 4 categories:
  - Global (6): coding-style, commenting, conventions, error-handling, tech-stack, validation
  - Backend (4): api, migrations, models, queries
  - Frontend (4): accessibility, components, css, responsive
  - Testing (1): test-writing
- Added comprehensive `docs/bert/standards/README.md` explaining standards system

**Agent Updates**
- Created `task-iterator.md` with conditional standards injection
- Created `task-executor.md` with conditional standards injection
- Standards injection controlled by `features.standards_injection` flag
- Agents read all 15 standards files when flag enabled
- Includes compliance requirements and conflict resolution

**Documentation**
- Created `docs/bert/standards/EXAMPLES.md` with usage examples (placeholder)
- Added migration guide for existing users (placeholder)
- Documented token usage trade-offs (estimated: +10-15k per task)
- Provided test results and verification

**Testing**
- Automated test suite verifies backward compatibility (4/4 tests passed)
- Directory structure matches Agent-OS 2.1 layout
- All standards files created with boilerplate content adapted from AOS

**Feature Flag**
- `features.standards_injection: false` (default - maintains backward compatibility)
- Users opt-in by setting to `true`
- All existing workflows work unchanged when disabled

### Impact

- **Backward Compatible**: 100% - no breaking changes
- **Opt-in**: Standards disabled by default, users enable when ready
- **Agent-OS Compatible**: Directory structure and approach match AOS 2.1
- **Token Usage**: +10-15k estimated per task, offset by reduced iteration cycles

### References

- Spec-27: Phase 1 - Standards Adoption (`docs/bert/specs/spec-27-p1sa/`)
- Test Results: `docs/bert/specs/spec-27-p1sa/TEST-RESULTS.md`
- Agent-OS 2.1 compatibility maintained

## [0.2.1] - 2025-10-26

### Added - Phase 0: Foundation for Agent-OS Harmony

**Compatibility Infrastructure**
- Added version tracking to skill.yml (`version: "0.2.1"`)
- Added compatibility metadata (`compatibility.agent_os_version: "2.1"`)
- Added feature flag system (all disabled by default):
  - `features.standards_injection: false` (Phase 1)
  - `features.workflow_modularization: false` (Phase 2)
  - `features.frontmatter_agents: false` (Phase 3)
- Added path configurations for future phases:
  - `paths.standards_directory: docs/bert/standards`
  - `paths.workflows_directory: .claude/skills/bert/workflows`

**Documentation**
- Created spec-26: Phase 0 - Foundation (`docs/bert/specs/spec-26-a2iew/`)
- Created ARCHITECTURE.md: 6-phase integration roadmap
- Created spec-27 stub: Phase 1 - Standards Adoption
- Created spec-28 stub: Phase 2 - Workflow Modularization
- Created spec-29 stub: Phase 3 - Agent Organization

**Impact**
- Zero user impact - all changes are metadata/configuration
- 100% backward compatibility maintained
- All existing workflows work unchanged
- Feature flags enable gradual opt-in for future phases

**Next Steps**
- Phase 1 (spec-27): Standards Adoption (3-4 weeks)
- Phase 2 (spec-28): Workflow Modularization (4-6 weeks)
- Phase 3 (spec-29): Agent Organization (6-8 weeks)
- Phases 4-6: Feature Flags, Bridge, Future-Proofing (later)

**References**
- Spec-26: `docs/bert/specs/spec-26-a2iew/requirements.md`
- Architecture: `docs/bert/specs/spec-26-a2iew/ARCHITECTURE.md`
- Analysis: `docs/bert/os-bert.md`

### Added - Rust CLI

- Rust CLI (`bert`) for fast task operations from command line
  - `bert task stub` - Create task stubs with parent-child support
  - `bert task archive` - Archive tasks with recursive children option
  - `bert spec stub` - Create spec stubs
  - `bert spec archive` - Archive specs and related tasks
  - Universal numbering system (scans tasks + specs across active/archive)
  - Project root detection from any subdirectory
  - Full integration with existing `skill.yml` configuration
- Comprehensive test suite (59 tests: 52 unit + 7 integration)
- Rust CLI documentation in README

### Changed
- Version bumped to 0.2.1 to reflect Rust CLI and Phase 0 compatibility infrastructure
- README updated with Rust CLI installation and usage instructions

## [0.1.3] - 2025-10-22

### Added
- Task execution command (`/bert/task`) with enhanced documentation
- Detailed walkthrough guide (`docs/walkthrough.md`) with comprehensive usage examples
- Requirements gatherer agent v2 with improved functionality
- Archive documentation to spec command

### Changed
- Revised interaction cadence to provide more user control
- Updated skill documentation with 261 new lines of detailed instructions
- Improved spec iterator agent with streamlined workflow
- Enhanced README with task command documentation and examples
- Expanded walkthrough with task execution examples

### Fixed
- Repository name references in README and installation scripts
- Spec command documentation and structure

## [0.0.1] - 2025-10-20

### Added
- Initial release
- Base installation scripts
- Core BERT functionality
- Basic README documentation
