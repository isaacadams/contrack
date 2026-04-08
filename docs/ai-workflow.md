# AI Workflow

This guide explains how to use AI with Contrack to identify meaningful contributions made by a specific developer.

The core idea is simple:

1. Use `contrack` to import and store commit evidence.
2. Use `gh` to gather higher-signal GitHub context such as pull request descriptions.
3. Have AI group related commits into larger, outcome-oriented contributions.
4. Save accepted contributions back into Contrack.

## Goal

The AI should identify larger contributions made by one target user, not produce a flat list of commits.

Good contributions usually represent one of these:

1. a feature
2. a feature enhancement
3. a bug-fix effort
4. a refactor
5. infrastructure or reliability work
6. performance work
7. meaningful tooling improvements

## Tool Roles

### Use `contrack` for

1. refreshing imported commit evidence
2. listing commits already stored in the contribution ledger
3. saving accepted contributions
4. generating final markdown output

Recommended commands:

```bash
contrack refresh --repo <repo>
contrack commit list --repo <repo> --limit 100
contrack contribution add ...
contrack contribution list --repo <repo>
contrack generate markdown --repo <repo> --style resume
```

### Use `gh` for

1. finding pull requests authored by the target user
2. reading PR titles and descriptions
3. validating authorship and scope
4. checking changed files and commit lists for a PR

Recommended commands:

```bash
gh pr list --author <user> --state all
gh pr view <number> --json title,body,author,files,commits
gh search prs --author <user> --repo <owner/repo>
```

## Recommended Workflow

### 1. Refresh evidence in Contrack

Start by refreshing commit metadata for the repository so the local ledger is current.

```bash
contrack refresh --repo <repo>
```

### 2. Inspect recent commit evidence

Look at the imported commit set before grouping anything.

```bash
contrack commit list --repo <repo> --limit 100
```

### 3. Fetch GitHub context for the target user

Commit messages alone are often too noisy. Use PR context to understand the real outcome.

```bash
gh pr list --author <user> --state all
gh pr view <number> --json title,body,author,files,commits
```

Good evidence to use:

1. PR title
2. PR description
3. PR author
4. files changed
5. commit list
6. linked issue or ticket references

### 4. Group commits into contribution candidates

The AI should group related commits into a larger contribution when they clearly support one outcome.

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

### 5. Be conservative

Prefer fewer, stronger contributions.

Down-rank or ignore:

1. formatting-only changes
2. dependency churn without clear impact
3. isolated housekeeping
4. noisy WIP commits unless they are part of a larger finished outcome
5. tiny docs or typo fixes unless they materially changed the product or workflow

### 6. Save accepted contributions in Contrack

Once the grouping looks right, save the contribution with `contrack contribution add`.

Example:

```bash
contrack contribution add \
  --repo <repo> \
  --name "Contribution identification workflow" \
  --overview "Built a repeatable workflow for turning commit and PR evidence into structured contributions." \
  --description "Combined imported commit metadata with GitHub pull request context so related work could be grouped into larger, high-signal contribution records instead of noisy per-commit logs." \
  --category "Tooling" \
  --priority 4 \
  --key-commit abc1234 \
  --related-commit def5678 \
  --technical-detail "Uses `gh pr view` data to validate commit grouping and authorship" \
  --resume-bullet "Turned noisy git and PR history into reusable contribution records for career materials"
```

## Attribution Rules

When identifying contributions for a specific user:

1. start with commit author evidence
2. validate with PR authorship where possible
3. keep only clearly attributable work if multiple people contributed
4. mark uncertain groupings as uncertain rather than forcing them into the ledger

## What the AI Should Optimize For

The AI should optimize for contributions that are useful for:

1. resumes
2. portfolios
3. performance reviews
4. project summaries

That means the final contribution records should describe outcomes, not just implementation activity.

## Prompt Template

Use this prompt for an AI agent that has access to `contrack` and `gh`.

```text
You are identifying meaningful engineering contributions made by a specific user in a repository.

Primary tools:
- Use `contrack` as the source of record for imported commits and saved contributions.
- Use `gh` to fetch GitHub context such as pull request titles, descriptions, authors, files, and commit lists.

Goal:
Turn noisy commit history into a small set of high-signal contributions for one specific contributor.

Rules:
- Do not create one contribution per commit.
- Group related commits into larger projects, features, fixes, refactors, infrastructure work, reliability work, performance work, or tooling improvements.
- Prefer contributions that are meaningful for resumes, portfolios, and performance reviews.
- Use PR descriptions and related GitHub metadata when commit messages alone are too noisy.
- Be conservative: fewer strong contributions are better than many weak ones.
- If attribution is mixed or unclear, keep only the clearly attributable part or mark the grouping as uncertain.

Process:
1. Refresh commit evidence with `contrack refresh` or inspect current evidence with `contrack commit list`.
2. Identify commits by the target user.
3. Use `gh` to fetch related PR context where available.
4. Group commits into candidate contributions.
5. Draft structured contribution records.
6. Save accepted contributions with `contrack contribution add`.

For each candidate contribution, provide:
- name
- overview
- description
- category
- priority
- key commits
- related commits
- technical details
- resume bullets
- rationale

Favor outcome-oriented wording, not raw implementation logs.
```

## Review Checklist

Before saving a contribution, verify:

1. the work belongs to the target user
2. the grouped commits clearly serve one outcome
3. the contribution is meaningful enough to keep
4. the name and description describe impact, not noise
5. the selected key commits are representative
