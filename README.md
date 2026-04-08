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
  --status accepted \
  --confidence high \
  --covered-pr 123 \
  --key-commit abc1234 \
  --related-commit def5678 \
  --rationale "Grouped the core refactor PRs into one product-level V1 milestone." \
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
contrack repo status [repo] [--json]
contrack repo remove <repo>
```

Repository selectors accept a slug, name, local path, or remote URL.

### Contributions

```bash
contrack contribution add --name <name> --overview <text> --description <text> --key-commit <hash>...
contrack contribution edit <id-or-name> [--name <name>] [--overview <text>] [--description <text>]
contrack contribution link-pr <id-or-name> <pr> [<pr>...] [--replace]
contrack contribution merge <primary-id> <secondary-id>
contrack contribution list [--repo <repo>] [--json]
contrack contribution show <id-or-name> [--json]
```

Useful contribution flags:

```bash
--category <category>
--priority <1-5>
--status <draft|accepted>
--confidence <high|medium|low>
--rationale <text>
--covered-pr <number>
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
contrack commit list [--repo <repo>] [--contribution <id-or-name>] [--limit <n>] [--json]
contrack commit authors [--repo <repo>] [--limit <n>] [--json]
```

`refresh` is the daily “pull the latest evidence into Contrack” command. `update` is available as an alias.

If `refresh` fails because the repository is not tracked yet, Contrack now points you toward `contrack repo add <path> --slug <slug>`.

### Markdown Generation

```bash
contrack generate markdown [--repo <repo>] [--style resume|portfolio] [--output <file>] [--status <draft|accepted>] [--include <id,id,...>] [--json]
```

## Data Model

Each contribution stores:

1. `name`
2. `overview`
3. `description`
4. `category`
5. `priority`
6. `status`
7. `confidence`
8. `rationale`
9. `covered PRs`
10. `key commits`
11. `related commits`
12. `technical details` (optional)
13. `resume bullets` (optional)

Commits are imported from git with:

1. hash
2. author
3. date
4. message summary
5. files changed
6. lines added and deleted

Commit hashes in contributions are matched against imported commits by exact hash or unique prefix.

Contrack does not currently import pull request metadata directly.

Instead, use `gh` to inspect pull requests and then save the relevant PR numbers to contributions with `--covered-pr`.

Contribution records can store PR links from external evidence such as:

1. number
2. title and scope discovered via `gh`
3. author login and name validated with `gh`
4. state and merged date validated with `gh`

## Database Locations

Contrack prefers a project-local database when you run `contrack init`:

- Project-local: `<repo>/.contrack/contrack.db`
- Global fallback:
  - Linux: `~/.local/share/contrack/contrack.db`
  - macOS: `~/Library/Application Support/com.contrack.contrack/contrack.db`
  - Windows: `%APPDATA%\contrack\contrack.db`

Run `contrack locations` to see the active path on your machine.

Existing SQLite databases are upgraded automatically when new Contrack versions add columns or tables.

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
2. `gh` is the tool for pull request exploration, titles, descriptions, authorship, files, and commit context.
3. The AI should group related commits and PRs into larger contributions, not create one contribution per commit.
4. The AI should inspect the active Contrack environment before taking any action.
5. The AI should present contribution candidates for approval before writing anything to Contrack.
6. The AI should build a full meaningful contribution inventory for the target user, not a shortlist.
7. The resulting catalog should act like a reusable snapshot of that developer's repo-level work for interviews, resumes, promotions, and performance reviews.

Use `gh` first, then `git log` only for gaps or validation.

Recommended workflow for one target user:

```bash
# 1. Check the active Contrack environment first.
contrack locations
contrack repo list

# Inputs to provide to the AI up front:
# - target GitHub login
# - target commit author/display name

# 2. Refresh imported commit evidence if the repo is already tracked.
contrack refresh <repo>

# Do not run `contrack repo add` unless the user explicitly approved it.
# Repositories explicitly listed in the original request count as approved.

# 3. Inspect stored commit evidence and saved contributions.
contrack commit list --repo <repo> --limit 100 --json
contrack commit authors --repo <repo> --limit 20 --json
contrack contribution list --repo <repo> --json
contrack contribution show <id-or-name> --json

# 4. Fetch pull request evidence directly with gh.
gh pr list --repo <owner/repo> --author <github-user> --state all --limit 100
gh pr view <number> --repo <owner/repo> --json title,body,author,files,commits

# Prefer merged PRs for durable evidence.
# If author lookup returns nothing useful, verify the GitHub login and retry once.
# If needed, cross-check once with: gh search prs --author <github-user> --repo <owner/repo>

# 5. Use targeted git log only to fill gaps.
git log --author='Isaac Adams' --date=short --pretty=format:'%h %ad %s'

# Exclude open PRs by default unless in-flight work is explicitly requested.
# If multiple repos are in scope, analyze each repo separately first.

# 6. Manually check coverage using:
# - workstream
# - disposition: keep / merge / exclude
# - linked candidate
# - exclusion reason if excluded
# 7. Present grouped candidates for approval.
# 8. Save curated contribution records with contrack after approval.
contrack contribution add --help
contrack contribution edit --help
contrack contribution link-pr --help

# 9. Generate final markdown after approval.
contrack generate markdown --repo <repo> --style resume --status accepted
```

When using AI, instruct it to:

1. Identify contributions made by one specific user.
2. Require both the target GitHub login and the target commit author/display name when possible.
3. Group related commits into larger projects, features, refactors, fixes, or infrastructure efforts.
4. Review existing saved contributions before proposing new ones.
   Prefer refining, merging, or extending saved candidates over creating duplicates.
5. Use `gh` when commit messages alone are too noisy.
6. Start with merged PRs from `gh pr list` and `gh pr view`, then use narrow `git log` queries only when PR evidence is missing or ambiguous.
7. Exclude mixed-authorship work unless the target user's portion is clearly separable.
8. Exclude open PRs by default unless in-flight work is requested.
9. Use `covered_prs`, `confidence`, `rationale`, and `status` when saving contribution records.
10. Use the live CLI help as the source of truth for exact save/edit flags instead of relying on copied examples.
11. Save accepted contributions with `contrack contribution add` only after explicit approval.
12. Build a full meaningful contribution inventory, not a shortlist.
13. Run a manual coverage check so every meaningful workstream is saved, merged, or intentionally excluded.
14. Still group related work into larger outcomes rather than producing a changelog.
15. Treat the saved contribution catalog like a durable snapshot of the developer's work in that repository.
16. If multiple repos are requested, analyze each repo separately first and only then provide cross-repo synthesis.

Detailed guidance and prompt templates live in `docs/ai-workflow.md`.

## Development

Run the standard verification steps:

```bash
cargo build
cargo build --release
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```
