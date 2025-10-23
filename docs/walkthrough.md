# Bert Walkthrough: Building User Authentication

Complete step-by-step guide from installation to production-ready feature.

---

## Installation

**Step 1: Install Bert**

```bash
cd your-project
curl -fsSL https://raw.githubusercontent.com/ssr1ram/bert/main/scripts/base-install.sh | bash
```

This creates:
- `.claude/commands/bert/` - Command definitions
- `.claude/skills/bert/` - Skill logic and agents
- `docs/bert/` - Where your work lives

**Step 2: (Optional) Set up product context**

```bash
/bert:plan
```

This creates templates in `docs/bert/product/`:
- `mission.md` - Your product's mission
- `roadmap.md` - Completed features and plans
- `tech-stack.md` - Technologies you use

Fill these out to help AI understand your project better. **Skip if you just want to start coding.**

---

## Scenario: Add User Authentication

Let's build a complete authentication system with email login, session management, and protected routes.

---

## Step 1: Create Specification

**Run:**
```bash
/bert:spec new "user authentication system with email login and protected routes"
```

**What happens:**
- AI creates `docs/bert/specs/spec-01/requirements.md`
- File opens with targeted questions
- AI creates `docs/bert/specs/spec-01/visuals/` for mockups

**What you see in requirements.md:**

```markdown
# Spec 01: User Authentication System - Requirements

## 1. Authentication Method
What authentication method do you want to use?
- [ ] Email/password
- [ ] Magic link (passwordless)
- [ ] OAuth (Google, GitHub, etc.)
- [ ] Multi-factor authentication

**Your answer:**

## 2. Session Management
How should sessions be handled?
...
```

---

## Step 2: Answer Requirements

**Edit `docs/bert/specs/spec-01/requirements.md`:**

```markdown
## 1. Authentication Method
What authentication method do you want to use?
- [x] Magic link (passwordless)

**Your answer:**
Email-only authentication with magic links. No passwords to manage.
Users click link in email to sign in.

## 2. Session Management
How should sessions be handled?

**Your answer:**
JWT tokens stored in httpOnly cookies. 7-day expiration.
Refresh tokens for seamless re-auth.

## 3. Protected Routes
Which routes need authentication?

**Your answer:**
- `/dashboard` - User dashboard
- `/settings` - Account settings
- `/api/user/*` - All user API endpoints

Public routes: `/`, `/login`, `/about`
```

**Add mockup (optional):**
- Drop `login-flow.png` into `docs/bert/specs/spec-01/visuals/`

---

## Step 3: Generate Specification

**Run:**
```bash
/bert:spec iterate 1
```

**What happens:**
- AI reads your answers
- May ask follow-up questions OR generate spec
- If follow-ups added, answer them and run `/bert:spec iterate 1` again

**After iteration, you get `spec.md`:**

```markdown
# Spec 01: User Authentication System

## Goal
Implement passwordless authentication using magic links...

## Technical Approach
- **Auth Provider**: InstantDB auth
- **Email Service**: Resend
- **Session Storage**: httpOnly JWT cookies
...

## Database Schema
...

## API Endpoints
...

## Components
...
```

**Review and iterate:**

If you want changes, add feedback at the bottom of `spec.md`:

```markdown
---

## Feedback

- Add rate limiting for magic link requests
- Need password reset flow for existing users
```

Then run:
```bash
/bert:spec iterate 1
```

AI updates the spec based on your feedback. **Repeat until satisfied.**

---

## Step 4: Generate Tasks

**Run:**
```bash
/bert:spec tasks 1
```

**What happens:**
- AI analyzes the spec
- Creates task breakdown in `docs/bert/specs/spec-01/tasks-proposal.md`
- You review the proposal

**tasks-proposal.md looks like:**

```markdown
# Task Breakdown: User Authentication

## Proposed Tasks

### 01.1: Database Schema & Models
Set up user, session, and magic link tables...

### 01.2: Magic Link Generation
API endpoint to generate and email magic links...

### 01.3: Email Verification & Login
Verify magic link tokens and create sessions...

### 01.4: Auth Middleware
Protect routes with JWT verification...

### 01.5: Frontend Login UI
Login form and magic link sent confirmation...

### 01.6: Session Management
Refresh token logic and logout...

## Dependencies
01.2 depends on 01.1
01.3 depends on 01.2
01.4 depends on 01.3
```

**Review and approve:**

If good, AI creates actual task files:
```
docs/bert/tasks/
├── task-01.1-database-schema-models.md
├── task-01.2-magic-link-generation.md
├── task-01.3-email-verification-login.md
├── task-01.4-auth-middleware.md
├── task-01.5-frontend-login-ui.md
└── task-01.6-session-management.md
```

Each file has:
- Description
- Acceptance criteria
- Reference to parent spec

---

## Step 5: Execute Tasks

### Execution Options

Bert supports multiple execution modes:

**Single task:**
```bash
/bert:task execute 1.1
```

**Range of tasks (sequential):**
```bash
/bert:task execute 1.1 to 1.6
```
This executes all tasks from 1.1 through 1.6 automatically, with dependency checking and progress tracking.

**Multiple specific tasks:**
```bash
/bert:task execute 1.1 1.3 1.5
```

### Example: Execute First Task

**Start with first task:**

```bash
/bert:task execute 1.1
```

**What happens:**

1. **AI checks dependencies** - Warns if dependent tasks aren't completed
2. **AI updates status** - Changes task from `pending` → `in-progress`
3. **AI reads the task** (`task-01.1-database-schema-models.md`)
4. **AI implements it:**
   - Creates database migration
   - Defines User, Session, MagicLink models
   - Sets up relationships
   - Runs build to verify
5. **AI verifies success criteria** - Checks all acceptance criteria from task file
6. **AI updates status** - Changes task to `completed`
7. **AI creates review file** (`task-01-review.md`):

```markdown
# Task 01: User Authentication System - Review

**Created**: 2025-01-21
**Status**: In Review

---

## Implementation Summary

### Completed Tasks
- [x] 01.1: Database Schema & Models

### Files Changed
- src/db/migrations/001_create_auth_tables.sql
- src/db/models/User.ts
- src/db/models/Session.ts
- src/db/models/MagicLink.ts
- src/db/schema.ts

### Implementation Notes
Created three tables: users, sessions, magic_links.

Users table:
- id (uuid primary key)
- email (unique, not null)
- created_at, updated_at

Sessions table:
- JWT tokens stored as hashed values
- 7-day expiration as specified
- Foreign key to users

Magic links:
- Token expires in 15 minutes
- Single-use (deleted after verification)
- Rate limited to 3 per email per hour

---

## Testing Checklist

- [x] Build succeeds without errors
- [ ] Database migration runs successfully
- [ ] Models follow TypeScript conventions
- [ ] Relationships are correctly defined
- [ ] Tested schema locally

---

## Issues Found

<!-- Add issues here -->

---

## Final Status

- [ ] Ready for production
- [x] Needs iteration

**Notes**: Task 01.1 complete. Ready for review.
```

4. **AI tells you:**
   > "Created review file at docs/bert/tasks/task-01-review.md.
   > Please test the implementation and add any issues to the Issues Found section.
   > When done, tell me: 'added notes to task-01-review.md'"

---

## Step 6: Test & Report Issues

**You test the implementation:**

```bash
# Run migration
npm run db:migrate

# Check tables
psql -d mydb -c "\dt"
# ❌ Error: magic_links table missing 'user_id' column
```

**You find issues and add them to task-01-review.md:**

```markdown
## Issues Found

### Issue 1: Missing user_id in magic_links table (2025-01-21)
**Reporter**: User
**Status**: Open

The magic_links table doesn't have a user_id foreign key.
Can't associate magic links with users.

Migration should include:
user_id UUID REFERENCES users(id) ON DELETE CASCADE

### Issue 2: Sessions table missing device info (2025-01-21)
**Reporter**: User
**Status**: Open

For security, we should track device information:
- user_agent
- ip_address
- last_active timestamp
```

---

## Step 7: Notify AI to Fix

**Tell AI:**

```
added notes to task-01-review.md
```

**What happens:**

1. **AI reads the review file**
2. **AI sees your issues**
3. **AI fixes each issue:**
   - Updates migration file
   - Adds user_id to MagicLink model
   - Adds device tracking to Session model
   - Runs build
4. **AI documents fixes in review file:**

```markdown
### Issue 1: Missing user_id in magic_links table (2025-01-21)
**Reporter**: User
**Status**: Fixed

The magic_links table doesn't have a user_id foreign key.
Can't associate magic links with users.

Migration should include:
user_id UUID REFERENCES users(id) ON DELETE CASCADE

**Fix**: Updated src/db/migrations/001_create_auth_tables.sql to add:
- user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE
Also updated src/db/models/MagicLink.ts to include userId relation.

### Issue 2: Sessions table missing device info (2025-01-21)
**Reporter**: User
**Status**: Fixed

For security, we should track device information...

**Fix**: Updated Session model and migration to include:
- userAgent TEXT
- ipAddress INET
- lastActiveAt TIMESTAMP DEFAULT NOW()
Added index on (userId, lastActiveAt) for efficient queries.
```

5. **AI tells you:**
   > "Fixed 2 issues in task-01-review.md:
   > - Added user_id to magic_links table
   > - Added device tracking to sessions table
   >
   > All fixes documented in review file. Please test again."

---

## Step 8: Verify & Approve

**You test again:**

```bash
npm run db:migrate
# ✓ Success

npm run build
# ✓ No errors

psql -d mydb -c "\d magic_links"
# ✓ user_id column present
```

**Everything works! Update review file:**

```markdown
## Final Status

- [x] Ready for production
- [ ] Needs iteration

**Notes**: All issues resolved. Migration tested. Schema validated.
```

---

## Step 9: Execute Remaining Tasks

**Execute all remaining tasks at once using range syntax:**

```bash
/bert:task execute 1.2 to 1.6
```

**What happens:**
- AI executes tasks sequentially from 1.2 through 1.6:
  - 1.2: Magic link generation endpoint
  - 1.3: Email verification & login logic
  - 1.4: Auth middleware for protected routes
  - 1.5: Frontend login UI components
  - 1.6: Session management & logout
- AI checks dependencies before each task
- AI updates task status automatically (pending → in-progress → completed)
- AI updates **same review file** (task-01-review.md) with all changes
- Runs build after completing all tasks
- Provides execution summary with completion status

**Execution Summary:**

When executing multiple tasks, Bert provides a summary report:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Execution Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Completed: 5 tasks
Failed: 0 tasks

✅ Task 01.2: Magic Link Generation
✅ Task 01.3: Email Verification & Login
✅ Task 01.4: Auth Middleware
✅ Task 01.5: Frontend Login UI
✅ Task 01.6: Session Management
```

**After execution completes:**

AI tells you:
> "Completed tasks 1.2-1.6. Updated task-01-review.md with all implementation details.
> Please test the complete authentication system and add any issues to the review file.
> When done, tell me: 'added notes to task-01-review.md'"

The review file now tracks the **entire feature**:

```markdown
# Task 01: User Authentication System - Review

### Completed Tasks
- [x] 01.1: Database Schema & Models
- [x] 01.2: Magic Link Generation
- [x] 01.3: Email Verification & Login
- [x] 01.4: Auth Middleware
- [x] 01.5: Frontend Login UI
- [x] 01.6: Session Management

### Files Changed
- src/db/migrations/001_create_auth_tables.sql
- src/db/models/User.ts
- src/api/auth/magiclink.ts
- src/api/auth/verify.ts
- src/middleware/auth.ts
- src/components/LoginForm.tsx
- src/hooks/useAuth.ts
... (30 files total)

### Implementation Notes
Built complete passwordless auth system using magic links.
Email delivery via Resend API. JWT sessions in httpOnly cookies.
Rate limiting: 3 magic links per email per hour.
Protected routes: /dashboard, /settings, /api/user/*

---

## Issues Found

### Issue 1: ... (Fixed)
### Issue 2: ... (Fixed)
### Issue 3: ... (Fixed)
...
### Issue 12: ... (Fixed)

---

## Final Status

- [x] Ready for production

**Notes**: All 6 tasks complete. 12 issues found and fixed.
Feature tested end-to-end. Ready to deploy.
```

---

## Step 10: Archive When Done

**After feature is in production, you have two options:**

### Option A: Archive Everything (Spec + Tasks)

```bash
/bert:spec archive 1
```

**What happens:**
- Archives spec directory: `docs/bert/specs/spec-01/` → `docs/bert/archive/specs/`
- Archives all task files: `task-01.1.md`, `task-01.2.md`, etc. → `docs/bert/archive/tasks/`
- Archives review file: `task-01-review.md`
- Archives any notes files

**Archive output:**
```
Archived spec 1:
- Spec directory archived: spec-01
- Tasks archived: 6
- Notes archived: 0
- Review archived: 1

Spec moved to: docs/bert/archive/specs/spec-01/
Tasks moved to docs/bert/archive/tasks/:
  task-01.1-database-schema-models.md
  task-01.2-magic-link-generation.md
  task-01.3-email-verification-login.md
  task-01.4-auth-middleware.md
  task-01.5-frontend-login-ui.md
  task-01.6-session-management.md
  task-01-review.md
```

### Option B: Archive Tasks Only (Keep Spec as Documentation)

```bash
/bert:spec archive 1 --tasks-only
```

**What happens:**
- Archives task files only
- **Keeps** spec directory at `docs/bert/specs/spec-01/`
- Useful when spec serves as living documentation

**Archive output:**
```
Archived tasks for spec 1:
- Spec directory: NOT archived (kept as documentation)
- Tasks archived: 6
- Review archived: 1

Tasks moved to docs/bert/archive/tasks/:
  task-01.1-database-schema-models.md
  ...
  task-01-review.md

Spec remains at: docs/bert/specs/spec-01/
```

### Option C: Archive Tasks Without Spec Context

**If you just want to archive tasks (e.g., for ad-hoc tasks):**

```bash
/bert:task archive 1
```

Works the same as Option B but doesn't require a spec to exist.

---

## Alternative: Ad-hoc Task (No Spec)

**For simple features, skip the spec:**

```bash
# Create task directly
/bert:task create "add logout button to navbar"

# Execute
/bert:task execute 15

# AI creates task-15-review.md
# Same review workflow
# Test, add issues, AI fixes
```

---

## Key Takeaways

1. **Specs are optional** - Use for complex features, skip for simple tasks
2. **Review workflow is automatic** - AI always creates review files
3. **Async collaboration** - Add issues when convenient, AI fixes on notification
4. **Single source of truth** - One review file per task/feature
5. **Iterative** - Keep testing and adding issues until ready

---

## Commands Reference

```bash
# Spec workflow
/bert:spec new "description"      # Create requirements
/bert:spec iterate <number>       # Generate/update spec
/bert:spec tasks <number>         # Create task files
/bert:spec archive <number>       # Archive spec + tasks
/bert:spec archive <number> --tasks-only  # Archive tasks, keep spec

# Task workflow
/bert:task create "description"   # Create ad-hoc task
/bert:task execute <number>       # Execute single task
/bert:task execute 1.1 to 1.6     # Execute range of tasks
/bert:task execute 1.1 1.3 1.5    # Execute specific tasks
/bert:task list                   # View all tasks
/bert:task status <number> <status> # Update status
/bert:task archive <number>       # Archive task files only

# Review workflow (automatic)
# 1. AI creates task-{NN}-review.md after execution
# 2. You add issues to "Issues Found" section
# 3. Tell AI: "added notes to task-{NN}-review.md"
# 4. AI fixes and documents in review file
# 5. Repeat until "Ready for production"
```

---

## Next Steps

- Try it with your own feature
- Read `README.md` for full command reference
- Check `.claude/skills/bert/skill.md` for advanced usage
- Join discussions at [GitHub repo](https://github.com/ssr1ram/bert)
