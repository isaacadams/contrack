# AI Workflow

Use AI with Contrack to build a full, reusable inventory of one developer's meaningful contributions to a repository.

Provide both of these inputs up front whenever possible:

1. target GitHub login
2. target commit author or display name

Default flow:

1. inspect the active Contrack environment
2. gather evidence with `contrack` and `gh`
3. group related commits and PRs into larger contributions
4. run a manual coverage check
5. present candidates for approval
6. save approved contributions

## Goal

The AI should identify larger contributions made by one target user, not produce a flat list of commits.

Treat the result as a durable snapshot of the developer's repository-level work. It should help the developer:

1. prepare for interviews
2. update a resume or portfolio
3. support promotion or performance-review packets
4. explain scope and ownership of past work quickly

Good contributions usually look like:

1. a feature
2. a feature enhancement
3. a bug-fix effort
4. a refactor
5. infrastructure or reliability work
6. performance work
7. meaningful tooling improvements

## Tool Roles

### Use `contrack` for

1. checking the active database and tracked repositories
2. refreshing imported commit evidence
3. listing commits and contributions already stored in the contribution ledger
4. saving accepted contributions after approval
5. generating final markdown output after approval

Core commands:

```bash
contrack locations
contrack repo list
contrack repo status [repo] --json
contrack commit list --repo <repo> --limit 100 --json
contrack commit authors --repo <repo> --limit 20 --json
contrack contribution list --repo <repo> --json
contrack refresh <repo>
contrack contribution add ...
contrack contribution link-pr <id-or-name> <pr> [<pr>...]
contrack generate markdown --repo <repo> --style resume
```

Prefer `--json` output from Contrack when it is available.

### Use `gh` for

1. finding pull requests authored by the target user
2. reading PR titles and descriptions
3. validating authorship and scope
4. checking changed files and commit lists for a PR

Command order:

1. `gh pr list --repo <owner/repo> --author <github-login> --state all --limit 100`
2. `gh pr view <number> --repo <owner/repo> --json title,body,author,files,commits`
3. `gh search prs --author <github-login> --repo <owner/repo>` only when you need to recover missed PRs or cross-check results

Use `gh search prs` only when you need to recover missed PRs or cross-check results.

## Recommended Workflow

### 1. Bootstrap the environment

Check which Contrack database is active and whether the target repository is already tracked.

```bash
contrack locations
contrack repo list
```

Do not run `contrack repo add` unless the user explicitly approves it. Repositories explicitly listed in the original request count as approved.

If the target repository is not already tracked, the AI should stop and ask for approval before running any state-changing command such as:

```bash
contrack init
contrack repo add .
contrack repo add /path/to/repo --slug <repo>
```

Do not assume the current repository is already registered, and do not silently add it.

### 2. Refresh evidence in Contrack

Refresh commit metadata so the local ledger is current.

```bash
contrack refresh <repo>
```

If the repository is not tracked yet, ask before refreshing.

### 3. Inspect recent commit evidence

Inspect recent imported commits before grouping anything.

```bash
contrack commit list --repo <repo> --limit 100 --json
```

Also check `contrack contribution list --repo <repo> --json` and `contrack contribution show <id-or-name> --json` so you do not duplicate saved work.

If author identity is unclear, use `contrack commit authors --repo <repo> --json` to inspect the commit author variants already stored in Contrack.

If saved contributions already exist, review and validate them before proposing new candidates. Prefer refining, merging, or extending existing saved candidates over creating duplicates.

### 4. Fetch GitHub context for the target user

Use PR context to understand the real outcome. Prefer merged PRs for durable contribution evidence, and use closed or unmerged PRs only as supporting context when needed.

```bash
gh pr list --repo <owner/repo> --author <github-login> --state all --limit 100
gh pr view <number> --repo <owner/repo> --json title,body,author,files,commits
```

Use the target GitHub login for `gh --author` queries, not the commit display name.

Prefer this evidence, in order:

1. PR title
2. PR description
3. PR author
4. files changed
5. commit list
6. linked issue or ticket references

PR evidence comes first. Raw `git log` is a gap-filling tool.

Use `git log` only when:

1. a meaningful commit cluster does not appear cleanly in PR results
2. PR metadata is missing or malformed
3. you need to validate a missing commit or ambiguous authorship claim

Keep fallback git queries narrow. Examples:

```bash
git log --author='Isaac Adams' --date=short --pretty=format:'%h %ad %s'
git log --grep='ENG-12345' --date=short --pretty=format:'%h %ad %s'
```

If command output is malformed or inconsistent:

1. do not continue as if the data is reliable
2. rerun the command once or use a narrower query
3. state the uncertainty explicitly in the analysis
4. avoid saving contributions based on corrupted or ambiguous evidence

If `gh pr list --author <github-login>` returns no useful results, verify the target GitHub login before continuing and retry once with the corrected login. If needed, cross-check once with `gh search prs --author <github-login> --repo <owner/repo>`.

### 5. Group commits into contribution candidates

Group related commits into one larger contribution when they clearly support one outcome.

Strong grouping signals:

1. same PR
2. same issue or ticket
3. same user-facing outcome
4. same subsystem or directory cluster
5. same time window and clearly progressive work

Split a candidate when:

1. one PR contains multiple distinct outcomes
2. commits are clearly unrelated
3. attribution to the target user is mixed or unclear

When mixed authorship is present:

1. do not attribute the full PR automatically to the target user
2. keep only the clearly attributable commits or clearly attributable portion of the outcome
3. exclude the candidate by default if the work cannot be separated cleanly

### 6. Be conservative

Prefer larger, meaningful contributions over fragmented or noisy entries.

Down-rank or ignore:

1. formatting-only changes
2. dependency churn without clear impact
3. isolated housekeeping
4. noisy WIP commits unless they are part of a larger finished outcome
5. tiny docs or typo fixes unless they materially changed the product or workflow

Interpret "all contributions" as all meaningful contributions worth storing in Contrack, not one contribution for every PR or every commit.

Exclude open PRs by default unless the user explicitly asks to include in-flight work.

The inventory should:

1. include the full meaningful contribution catalog
2. continue grouping related PRs and commits into larger outcomes
3. still ignore trivial housekeeping unless it materially changed behavior or operations
4. prefer a complete ledger of meaningful work, not an exhaustive changelog

### 7. Run a coverage check before saving

Before writing anything to Contrack, review the major workstreams you discovered and confirm each one is either:

1. saved as a contribution
2. merged into another contribution
3. intentionally excluded with a short reason

The manual coverage check should:

1. list the major workstreams discovered from PRs and commits
2. confirm each workstream is saved, merged into another contribution, or intentionally excluded
3. note the exclusion reason when a workstream is omitted
4. confirm there are no obvious uncovered PR clusters left unexplained

Present the manual coverage check using this structure:

1. workstream
2. disposition: keep, merge, or exclude
3. linked candidate
4. exclusion reason if excluded

Reasons for exclusion should be short and practical, such as:

1. housekeeping-only
2. too small
3. duplicate of larger contribution
4. attribution unclear

If multiple repositories are in scope, analyze each repository separately first. Cross-repo themes should be presented as a secondary synthesis, not as a replacement for the repo-level inventory.

### 8. Present candidates before saving

Before writing anything to Contrack, present candidates in a reviewable structure.

Use this format:

1. name
2. category
3. priority
4. key PRs
5. key commits
6. related commits
7. evidence summary
8. attribution confidence
9. rationale for grouping
10. exclusion reason when omitted
11. recommendation: keep, split, or discard

Wait for explicit approval before any write action. For large histories, present candidates in batches rather than saving as you go.

### 9. Save accepted contributions in Contrack

Once the grouping looks right and the user has approved the candidate, save it with `contrack contribution add`.

Use the CLI help as the source of truth for the exact save/edit command structure and supported flags:

```bash
contrack contribution add --help
contrack contribution edit --help
contrack contribution link-pr --help
```

Use `contrack contribution add` when creating a new saved contribution.

Use `contrack contribution edit` or `contrack contribution link-pr` when refining an existing saved contribution after review.

## Attribution Rules

When identifying contributions for a specific user:

1. start with commit author evidence
2. validate with PR authorship where possible
3. keep only clearly attributable work if multiple people contributed
4. mark uncertain groupings as uncertain rather than forcing them into the ledger

## State-Change Rules

By default, the AI should not run any of these without explicit approval:

1. `contrack init`
2. `contrack repo add`
3. `contrack contribution add`
4. `contrack contribution edit`
5. `contrack generate markdown`

This workflow is analysis and curation first.

The AI should also avoid unrelated state changes:

1. do not edit source files, docs, or config files during contribution identification
2. do not make git commits
3. do not run build, test, install, or formatting commands unless the user explicitly asks for them

When command syntax is unclear:

1. use `--help` once
2. update the approach based on the documented syntax
3. avoid repeated failed invocations of the same command shape
4. prefer the live CLI help over copied examples when exact flags or argument order matter

## What the AI Should Optimize For

Optimize for contributions that are useful for:

1. resumes
2. portfolios
3. performance reviews
4. project summaries
5. interview preparation
6. promotion packets

The final contribution records should describe outcomes, not just implementation activity. They should be accurate enough to defend in an interview and concise enough to reuse in career materials.

## Prompt Template

Use this prompt for an AI agent that has access to `contrack` and `gh`.

```text
You are identifying meaningful engineering contributions made by a specific user in a repository.

Inputs you should expect:
- Target GitHub login: <github-login>
- Target commit author/display name: <display-name>

Mode:
This is an analysis and curation task first. Do not modify repository state or Contrack state until I explicitly approve it.

Do not run without approval:
- `contrack init`
- `contrack repo add` unless the user explicitly listed the target repository in the original request
- `contrack contribution add`
- `contrack contribution edit`
- `contrack generate markdown`

Do not edit files, docs, or config as part of this task.
Do not make git commits.
Do not run build, test, install, or formatting commands.

Primary tools:
- Use `contrack` as the source of record for imported commits and saved contributions.
- Use `gh` to fetch GitHub context such as pull request titles, descriptions, authors, files, and commit lists.

Use the live CLI help as the source of truth for exact command shape when saving or editing data:
- `contrack contribution add --help`
- `contrack contribution edit --help`
- `contrack contribution link-pr --help`

Goal:
Turn noisy commit history into a full inventory of all meaningful contributions for one specific contributor.

Treat the result as a durable snapshot of that contributor's work in the repository, suitable for interviews, resumes, promotions, and performance reviews.

Startup sequence:
1. Run `contrack locations`.
2. Run `contrack repo list`.
3. Confirm whether the target repository is already tracked.
4. If the target repository is not tracked, stop and ask for approval before running `contrack init` or `contrack repo add`. Repositories explicitly listed in the original request count as approved for `contrack repo add`.
5. Refresh commit evidence with `contrack refresh <repo>` if needed.

Evidence order:
1. Use `gh pr list --repo <owner/repo> --author <github-login> --state all --limit 100`.
2. Use `gh pr view <number> --repo <owner/repo> --json title,body,author,files,commits`.
3. Use `contrack commit list --json`, `contrack contribution list --json`, and `contrack contribution show <id-or-name> --json` to inspect imported evidence and existing saved work.
   - Use `contrack commit authors --json` when you need to confirm commit-author naming.
4. Use `git log` only when PR evidence is insufficient.

Run `gh` first because PR descriptions, changed files, and commit lists usually provide better grouping evidence than raw commit chronology.
Prefer merged PRs for durable contribution evidence. Use closed or unmerged PRs only as supporting context when necessary.

Use the GitHub login for `gh --author` queries and the commit display name for targeted `git log --author` fallback queries.

Rules:
- Do not create one contribution per commit.
- Group related commits into larger projects, features, fixes, refactors, infrastructure work, reliability work, performance work, or tooling improvements.
- Prefer contributions that are meaningful for resumes, portfolios, and performance reviews.
- Prefer contributions the developer could realistically speak to and defend in an interview.
- Use PR descriptions and related GitHub metadata when commit messages alone are too noisy.
- Be conservative about noise, but thorough about meaningful work.
- If attribution is mixed or unclear, keep only the clearly attributable part or mark the grouping as uncertain.
- Exclude mixed-authorship PRs by default when the target user's portion cannot be separated cleanly.
- Interpret "all contributions" as all meaningful ledger-worthy contributions, not one contribution per PR or commit.
- Contribution candidates may span multiple PRs and long time windows when they clearly serve one coherent outcome.
- Each saved contribution must be materially distinct. If two candidates share the same central outcome, merge them or clearly separate their scope.
- Exclude open PRs by default unless the user explicitly asks to include in-flight work.
- Do not keep retrying the same invalid command pattern. If syntax is unclear, use `--help` once and correct course.
- If command output looks malformed or inconsistent, say so and verify it before using it as evidence.
- If author-based PR lookup returns no useful results, verify the GitHub login and retry once before falling back. If needed, cross-check once with `gh search prs --author <github-login> --repo <owner/repo>`.

Process:
1. Inspect the active Contrack environment and tracked repositories.
2. If the target repository is already tracked, refresh or inspect commit evidence with `contrack refresh <repo>`, `contrack commit list --repo <repo> --json`, and `contrack contribution list --repo <repo> --json`.
3. Identify PRs and commits by the target user.
4. Review any saved contributions before proposing new ones, and prefer refining, merging, or extending existing saved candidates over creating duplicates.
5. Use `gh` to fetch related PR context.
6. Group commits into candidate contributions.
7. If multiple repositories are in scope, complete repo-level analysis first and only then provide optional cross-repo synthesis.
8. Continue until the meaningful contribution inventory is covered.
9. Run a manual coverage check and confirm each meaningful cluster is saved, merged, or intentionally excluded.
10. Present grouped candidates for approval, using batches if the list is large.
11. Only after approval, save accepted contributions with `contrack contribution add`.

When refining an existing saved contribution after review, prefer `contrack contribution edit` or `contrack contribution link-pr` instead of recreating it from scratch.

Present the manual coverage check using this format:
- workstream
- disposition: keep, merge, or exclude
- linked candidate
- exclusion reason if excluded

Before saving anything, provide each candidate contribution in this format:
- name
- overview
- description
- category
- priority
- status
- covered_prs
- key PRs
- key commits
- related commits
- technical details
- resume bullets
- attribution confidence
- rationale
- exclusion reason if omitted
- recommendation: keep, split, or discard

Favor outcome-oriented wording, not raw implementation logs.
Wait for explicit approval before writing to Contrack.
```

## Review Checklist

Before saving a contribution, verify:

1. the work belongs to the target user
2. the grouped commits clearly serve one outcome
3. the contribution is meaningful enough to keep
4. the name and description describe impact, not noise
5. the selected key commits are representative
