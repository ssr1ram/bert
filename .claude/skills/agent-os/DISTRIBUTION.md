# Agent-OS Skill Distribution Guide

This guide explains how to package and share the agent-os skill with others.

## Distribution Methods

### Method 1: Git Repository (Recommended for Teams)

**Best for**: Teams working on the same codebase

1. **Include in your project repository**:
   ```bash
   git add .claude/skills/agent-os agent-os/
   git commit -m "Add agent-os skill"
   git push
   ```

2. **Team members get it automatically**:
   ```bash
   git clone <repository-url>
   # agent-os skill is already there!
   ```

3. **Optional: Use as a Git Submodule** (for skill-only repository):
   ```bash
   # Create a separate repo for just the skill
   git init agent-os-skill
   cd agent-os-skill
   cp -r /path/to/.claude/skills/agent-os ./
   cp -r /path/to/agent-os ./
   git add .
   git commit -m "Initial agent-os skill"
   git remote add origin <repository-url>
   git push -u origin main

   # Others can add it as a submodule
   git submodule add <repository-url> .claude/skills/agent-os
   ```

### Method 2: Archive File (Recommended for Distribution)

**Best for**: Sharing with external users or one-time distribution

1. **Create a distributable archive**:
   ```bash
   # From project root
   tar -czf agent-os-skill.tar.gz \
       .claude/skills/agent-os/ \
       agent-os/ \
       install-agent-os.sh \
       README-AGENT-OS.md
   ```

2. **Or create a zip file**:
   ```bash
   zip -r agent-os-skill.zip \
       .claude/skills/agent-os/ \
       agent-os/ \
       install-agent-os.sh \
       README-AGENT-OS.md
   ```

3. **Share the archive** via:
   - Email attachment
   - File sharing service (Dropbox, Google Drive, etc.)
   - GitHub Releases
   - Internal file server

4. **Users install by extracting**:
   ```bash
   # Extract to their project directory
   cd /path/to/their/project
   tar -xzf agent-os-skill.tar.gz
   # Or: unzip agent-os-skill.zip

   # Run installation verification
   bash install-agent-os.sh
   ```

### Method 3: GitHub Release

**Best for**: Public or versioned distribution

1. **Create a release on GitHub**:
   ```bash
   # Tag your version
   git tag -a v2.0.3 -m "Agent-OS skill v2.0.3"
   git push origin v2.0.3
   ```

2. **Attach the archive to the release**:
   - Go to GitHub → Releases → Create Release
   - Upload `agent-os-skill.tar.gz` as an asset
   - Write release notes

3. **Users download from releases page**:
   ```bash
   # Download the release
   wget https://github.com/<user>/<repo>/releases/download/v2.0.3/agent-os-skill.tar.gz

   # Extract and install
   tar -xzf agent-os-skill.tar.gz
   bash install-agent-os.sh
   ```

### Method 4: NPM Package (Advanced)

**Best for**: Integration with Node.js projects

1. **Create `package.json` for the skill**:
   ```json
   {
     "name": "@yourorg/agent-os-skill",
     "version": "2.0.3",
     "description": "Multi-agent specification and implementation system",
     "scripts": {
       "postinstall": "node scripts/install.js"
     },
     "files": [
       ".claude/skills/agent-os",
       "agent-os",
       "scripts/install.js"
     ]
   }
   ```

2. **Create install script** (`scripts/install.js`):
   ```javascript
   const fs = require('fs-extra');
   const path = require('path');

   async function install() {
     const projectRoot = path.resolve(process.cwd(), '../..');
     await fs.copy('.claude/skills/agent-os', path.join(projectRoot, '.claude/skills/agent-os'));
     await fs.copy('agent-os', path.join(projectRoot, 'agent-os'));
     console.log('✅ Agent-OS skill installed');
   }

   install().catch(console.error);
   ```

3. **Publish and install**:
   ```bash
   # Publish
   npm publish

   # Install in projects
   npm install @yourorg/agent-os-skill
   ```

## What to Include in Distribution

### Essential Files

```
.claude/skills/agent-os/
├── README.md                # Installation and usage guide
├── DISTRIBUTION.md          # This file
├── skill.yml                # Skill configuration
├── skill.md                 # Skill instructions
└── agents/                  # All 19 agent definitions
    └── *.md

agent-os/
├── product/                 # Can be empty or with examples
├── roles/
│   ├── implementers.yml
│   └── verifiers.yml
├── standards/               # Include all standards
└── specs/                   # Can be empty or with examples

install-agent-os.sh          # Installation script
README-AGENT-OS.md          # Top-level README (optional)
```

### Optional Files

- Example specifications in `agent-os/specs/`
- Example product docs in `agent-os/product/`
- Migration guide from old versions
- Changelog
- License file

## Pre-Distribution Checklist

- [ ] Verify all 19 agents are present in `.claude/skills/agent-os/agents/`
- [ ] Ensure `skill.yml` and `skill.md` are up to date
- [ ] Test installation script works
- [ ] Update version number in `skill.yml`
- [ ] Write clear README with examples
- [ ] Include LICENSE file
- [ ] Remove any sensitive/project-specific data from examples
- [ ] Test skill works in a fresh project
- [ ] Update CHANGELOG if applicable
- [ ] Tag version in git (if using version control)

## Testing Distribution Package

Before sharing, test your package:

```bash
# Create a test directory
mkdir /tmp/test-agent-os-install
cd /tmp/test-agent-os-install

# Extract your package
tar -xzf /path/to/agent-os-skill.tar.gz

# Run installation
bash install-agent-os.sh

# Verify structure
ls -la .claude/skills/agent-os
ls -la agent-os

# Test with Claude Code
# Open Claude Code and try: "plan the product"
```

## Version Control Best Practices

### For Skill Developers

```bash
# Track changes
git add .claude/skills/agent-os agent-os/
git commit -m "Update agent-os skill to v2.0.4"

# Tag versions
git tag -a v2.0.4 -m "Version 2.0.4 - Added new agents"
git push origin v2.0.4
```

### For Skill Users

```bash
# Keep specs out of version control (work-in-progress)
echo "agent-os/specs/*/" >> .gitignore

# But keep the directory structure
touch agent-os/specs/.gitkeep
git add agent-os/specs/.gitkeep
```

## Distribution Platforms

### Public Sharing

- **GitHub/GitLab**: Host as public repository
- **Anthropic Skills Marketplace**: (Future - when available)
- **NPM Registry**: For Node.js integration
- **Docker Hub**: As part of development containers

### Private Sharing

- **GitHub Private Repos**: For internal teams
- **GitLab Self-Hosted**: For enterprise
- **Artifactory/Nexus**: For artifact management
- **Shared Network Drive**: For local networks

## Maintenance & Updates

### Releasing Updates

1. Update version in `skill.yml`
2. Update CHANGELOG.md
3. Create git tag
4. Build new distribution package
5. Publish to distribution channels
6. Notify users

### User Updates

```bash
# Via git
git pull origin main

# Via archive
# Download new version and extract
tar -xzf agent-os-skill-v2.0.4.tar.gz

# Via npm
npm update @yourorg/agent-os-skill
```

## License Considerations

Choose an appropriate license:

- **MIT**: Permissive, allows commercial use
- **Apache 2.0**: Permissive with patent protection
- **ISC**: Similar to MIT, more concise
- **Proprietary**: For internal/commercial use only

Add a LICENSE file to your distribution.

## Support & Documentation

When distributing, provide:

- Installation instructions
- Usage examples
- Troubleshooting guide
- Contact information or issue tracker
- Contribution guidelines (if open source)

## Example Distribution README

```markdown
# Agent-OS Skill Distribution

Version: 2.0.3

## Quick Install

    tar -xzf agent-os-skill.tar.gz
    bash install-agent-os.sh

## Requirements

- Claude Code (2025+)
- Skills support (Pro/Max/Teams/Enterprise plan)

## Documentation

See `.claude/skills/agent-os/README.md` after installation.

## Support

- Issues: <repository-url>/issues
- Docs: <documentation-url>
- Email: <support-email>
```

## Common Distribution Mistakes to Avoid

1. **Don't include** `.git` directory in archives
2. **Don't include** work-in-progress specs with sensitive data
3. **Don't hardcode** absolute paths in scripts
4. **Don't forget** to test installation on a fresh system
5. **Don't skip** version numbers or changelog
6. **Don't distribute** without a license file

## Questions?

For distribution questions or issues, refer to:
- Claude Code Skills documentation
- This project's issue tracker
- Anthropic support channels
