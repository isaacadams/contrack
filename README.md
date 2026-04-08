# Contrack

Contrack is a focused personal contribution ledger for developers.

It helps you record meaningful code contributions, link them to git evidence, and generate clean Markdown you can reuse in resumes, portfolios, performance reviews, and project summaries.

> Contrack turns noisy git history into structured, high-signal contributions that you can reuse for your resume and portfolio.

## V1 Scope

Contrack V1 does five things well:

1. Track repositories
2. Import and refresh commit metadata from git
3. Track structured contributions
4. Query contributions, commits, stats, and database locations
5. Generate polished Markdown grouped by category and ordered by priority

## Installation

Build from source with Rust 1.78+:

```bash
git clone https://github.com/isaacadams/contrack.git
cd contrack
cargo build --release
./target/release/contrack --help
```

To install it into your Cargo bin directory:

```bash
cargo install --path .
```

## Quick Start

Initialize a project-local workspace:

```bash
contrack init
```

Track the current repository:

```bash
contrack repo add .
```

Import commit metadata:

```bash
contrack commit import
```

Add a contribution linked to commit evidence:

```bash
contrack contribution add \
  --name "Contribution ledger V1" \
  --overview "Refactored the CLI around repository, contribution, and markdown workflows." \
  --description "Removed AI/config bloat, rebuilt the schema, and shipped polished markdown generation for resume and portfolio use." \
  --category "Core Feature" \
  --priority 5 \
  --key-commit abc1234 \
  --related-commit def5678 \
  --technical-detail "SQLite schema centered on repositories, contributions, and imported commits" \
  --technical-detail "Git remote normalization supports SSH and HTTPS remotes" \
  --resume-bullet "Turned raw git history into structured, reusable contribution records" \
  --resume-bullet "Built resume-ready and portfolio-ready Markdown generation from stored contribution data"
```

Generate resume-style Markdown:

```bash
contrack generate markdown --style resume --output CONTRIBUTIONS.md
```

Generate portfolio-style Markdown to stdout:

```bash
contrack generate markdown --style portfolio
```

## Commands

### Workspace

```bash
contrack init
contrack locations
contrack stats [repo]
```

### Repositories

```bash
contrack repo add [path] [--name <display-name>] [--slug <slug>]
contrack repo list
contrack repo remove <repo>
```

Repository selectors accept a slug, name, local path, or remote URL.

### Contributions

```bash
contrack contribution add --name <name> --overview <text> --description <text> --key-commit <hash>...
contrack contribution edit <id-or-name> [--name <name>] [--overview <text>] [--description <text>]
contrack contribution list [--repo <repo>]
contrack contribution show <id-or-name>
```

Useful contribution flags:

```bash
--category <category>
--priority <1-5>
--key-commit <hash>
--related-commit <hash>
--technical-detail <text>
--resume-bullet <text>
```

`edit` replaces any repeated list field you pass and supports explicit clear flags:

```bash
--clear-key-commits
--clear-related-commits
--clear-technical-details
--clear-resume-bullets
```

### Commits

```bash
contrack commit import [repo] [--all]
contrack refresh [repo] [--all]
contrack commit list [--repo <repo>] [--contribution <id-or-name>] [--limit <n>]
```

`refresh` is the daily “pull the latest evidence into Contrack” command. `update` is available as an alias.

### Markdown Generation

```bash
contrack generate markdown [--repo <repo>] [--style resume|portfolio] [--output <file>]
```

## Data Model

Each contribution stores:

1. `name`
2. `overview`
3. `description`
4. `category`
5. `priority`
6. `key commits`
7. `related commits`
8. `technical details` (optional)
9. `resume bullets` (optional)

Commits are imported from git with:

1. hash
2. author
3. date
4. message summary
5. files changed
6. lines added and deleted

Commit hashes in contributions are matched against imported commits by exact hash or unique prefix.

## Database Locations

Contrack prefers a project-local database when you run `contrack init`:

- Project-local: `<repo>/.contrack/contrack.db`
- Global fallback:
  - Linux: `~/.local/share/contrack/contrack.db`
  - macOS: `~/Library/Application Support/com.contrack.contrack/contrack.db`
  - Windows: `%APPDATA%\contrack\contrack.db`

Run `contrack locations` to see the active path on your machine.

## Example Workflow

```bash
# 1. Initialize a local workspace inside the repository you care about.
contrack init

# 2. Track the repository.
contrack repo add .

# 3. Import commit evidence.
contrack refresh

# 4. Capture the contribution in structured form.
contrack contribution add \
  --name "Markdown generator" \
  --overview "Built high-signal contribution exports for career materials." \
  --description "Added grouped markdown output sorted by contribution priority so the same data can support both resume and portfolio writing." \
  --category "Feature" \
  --priority 4 \
  --key-commit 1a2b3c4d \
  --technical-detail "Resume and portfolio styles share one evidence model" \
  --resume-bullet "Created reusable markdown output from tracked engineering contributions"

# 5. Inspect what you have.
contrack contribution list
contrack contribution show 1
contrack commit list --contribution 1

# 6. Generate polished markdown.
contrack generate markdown --style portfolio --output PORTFOLIO_CONTRIBUTIONS.md
```

## AI-Assisted Contribution Identification

Contrack works best when AI helps curate contributions for a specific developer from commit and pull request evidence.

Use this division of labor:

1. `contrack` is the system of record for imported commit evidence and saved contributions.
2. `gh` is the context fetcher for pull request titles, descriptions, authorship, files, and linked metadata.
3. The AI should group related commits into larger contributions, not create one contribution per commit.

Recommended workflow for one target user:

```bash
# 1. Refresh imported commit evidence.
contrack refresh --repo <repo>

# 2. Inspect stored commits for the repository.
contrack commit list --repo <repo> --limit 100

# 3. Fetch pull request context for the target user.
gh pr list --author <github-user> --state all
gh pr view <number> --json title,body,author,files,commits

# 4. Save curated contribution records with contrack.
contrack contribution add ...

# 5. Generate final markdown.
contrack generate markdown --repo <repo> --style resume
```

When using AI, instruct it to:

1. Identify contributions made by one specific user.
2. Group related commits into larger projects, features, refactors, fixes, or infrastructure efforts.
3. Use `gh` when commit messages alone are too noisy.
4. Save accepted contributions with `contrack contribution add`.
5. Prefer fewer, stronger contributions over many weak ones.

Detailed guidance and prompt templates live in `docs/ai-workflow.md`.

## Development

Run the standard verification steps:

```bash
cargo build
cargo build --release
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
