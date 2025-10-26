# Changelog

All notable changes are documented here.

## [0.2.1] - 2025-10-26

### Added
- Rust CLI (`bert`) for fast task operations from command line
  - `bert task stub` - Create task stubs with parent-child support
  - `bert task archive` - Archive tasks with recursive children option
  - Universal numbering system (scans tasks + specs across active/archive)
  - Project root detection from any subdirectory
  - Full integration with existing `skill.yml` configuration
- Comprehensive test suite (59 tests: 52 unit + 7 integration)
- Rust CLI documentation in README

### Changed
- Version bumped to 0.2.1 to reflect new Rust CLI functionality
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
