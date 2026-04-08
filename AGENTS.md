# Development Rules and Guidelines for AI Agents

This document provides instructions for AI agents working on the contrack codebase. Follow these guidelines to ensure code quality and maintainability.

## Pre-Commit Verification

Before completing any changes, you MUST run the following commands to verify your work:

1. **Compilation Verification**

   ```bash
   cargo build
   cargo build --release
   ```

   - Both debug and release builds must succeed
   - Fix any compilation errors before proceeding

2. **Code Quality Check**

   ```bash
   cargo clippy
   ```

   - Fix all clippy warnings and errors
   - Ensure code follows Rust best practices

3. **Test Suite**

   ```bash
   cargo test
   ```

   - All existing tests must pass
   - No regressions in existing functionality
   - If tests fail, see "Test Failure Protocol" below

4. **Create Commit**
   - After all checks pass, create a commit of all changes made
   - Use descriptive commit messages that explain what was changed and why
   - Group related changes into logical commits

## Test Coverage

1. **Check Coverage**

   ```bash
   cargo-tarpaulin --out stdout
   ```

   - Review coverage report for gaps
   - Identify code paths without tests

2. **Add Missing Tests**
   - Add unit tests for any uncovered code
   - Ensure new code has comprehensive test coverage
   - Aim for high coverage of critical paths

## Unit Test Requirements

- **All new code MUST have unit tests**
- Tests should cover:
  - Happy paths
  - Error cases
  - Edge cases
  - Boundary conditions
- Test names should be descriptive and follow the pattern: `test_<functionality>`
- Place tests in the same file using `#[cfg(test)]` modules or in separate test files as appropriate

## Test Failure Protocol

If unit tests fail after your changes:

1. **First Attempt**: Analyze the failure, fix the issue, and re-run tests
2. **Second Attempt**: If tests still fail, review your changes more carefully, check for logic errors, and fix
3. **Third Attempt**: If tests continue to fail, review the test expectations and your implementation for fundamental misunderstandings
4. **Stop**: After 3 attempts, stop and document what was tried and what the issue appears to be

**Maximum 3 attempts** before stopping. Do not loop indefinitely.

## Regression Prevention

- Ensure existing functionality continues to work
- Run the full test suite before completing changes
- Verify that existing commands still function correctly
- Check that database migrations are backward-compatible where possible

## Code Quality Standards

- Follow Rust naming conventions
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused and single-purpose
- Handle errors appropriately using `Result` types
- Use `anyhow` for error handling consistency

## AI-Assisted Contribution Identification

When using Contrack itself to identify contributions for a specific developer, follow this workflow:

1. Start in analysis mode.
   - Do not edit files, docs, or config during contribution identification
   - Do not make git commits
   - Do not run build, test, install, or formatting commands unless explicitly asked
   - Do not run `contrack init`, `contrack repo add`, `contrack contribution add`, `contrack contribution edit`, or `contrack generate markdown` without explicit approval

2. Bootstrap the active Contrack environment first.
   - Run `contrack locations`
   - Run `contrack repo list`
   - Confirm whether the target repository is already tracked before attempting `refresh` or any write action
   - Do not run `contrack repo add` unless the user explicitly approved it
   - Repositories explicitly listed in the original request count as approved for `contrack repo add`

3. Use `contrack` as the system of record.
    - Refresh stored evidence with `contrack refresh <repo>` or `contrack commit import`
    - Prefer `--json` output when available
    - Inspect imported evidence with `contrack commit list`
    - Use `contrack commit authors` when commit-author identity is unclear
    - Check for existing saved work with `contrack contribution list`
    - Review saved contributions before proposing new candidates
    - Prefer editing or linking PRs onto existing contributions instead of recreating them from scratch
    - Save accepted contribution records with `contrack contribution add` only after approval

4. Use `gh` to fetch higher-signal GitHub context.
   - Prefer PR titles, descriptions, authorship, changed files, and commit lists over raw commit messages alone
   - Use the target GitHub login for `gh --author` queries, not the commit display name
   - Useful commands include:
      - `gh pr list --repo <owner/repo> --author <github-login> --state all --limit 100`
      - `gh pr view <number> --repo <owner/repo> --json title,body,author,files,commits`
      - `gh search prs --author <github-login> --repo <owner/repo>`
   - Prefer merged PRs for durable evidence; use closed or unmerged PRs only as supporting context when needed
   - If author lookup returns no useful results, verify the GitHub login and retry once before falling back; if needed, cross-check once with `gh search prs`

5. Prefer PR evidence over raw git chronology.
   - Start from author-matched PRs and their changed files, titles, descriptions, and commits
   - Use `git log` only to fill evidence gaps or validate missing commits
   - If command output is malformed or inconsistent, rerun or narrow the query before using it as evidence

6. Identify larger contributions, not raw commit logs.
    - Group related commits into one contribution when they clearly serve one outcome
    - Prefer features, major refactors, infrastructure work, reliability work, performance work, and meaningful tooling improvements
    - Avoid creating one contribution per commit unless a single commit fully represents a meaningful standalone accomplishment
    - Treat the final contribution set as a reusable snapshot of the target developer's repository-level work

7. Group commits using strong evidence.
   - Group by shared PR, feature, issue, subsystem, or tightly related time window
   - Split when one PR contains multiple distinct outcomes or when the work is clearly unrelated

8. Attribute carefully to a specific user.
   - Start with commit author evidence
   - Validate with PR authorship and PR context from `gh`
   - If authorship is mixed or unclear, keep only the clearly attributable portion or mark the grouping as uncertain
   - Exclude mixed-authorship PRs by default when the target user's portion cannot be separated cleanly

9. Be conservative.
    - Prefer larger, meaningful contributions over fragmented or noisy entries
    - Down-rank or ignore formatting-only changes, dependency churn without clear impact, isolated housekeeping, and noisy WIP commits unless they are part of a larger finished outcome
    - Interpret "all contributions" as all meaningful ledger-worthy contributions, not one contribution per PR or commit
    - Exclude open PRs by default unless the user explicitly asks to include in-flight work

10. Present candidate contributions for review before saving.
     - For each candidate, include: name, category, priority, status, covered PRs, key PRs, key commits, related commits, evidence summary, attribution confidence, rationale, exclusion reason when relevant, and keep/split/discard recommendation
     - For large histories, present candidates in batches rather than saving as you go

11. Handle multiple repositories deliberately.
    - Analyze each repository separately first
    - Present cross-repo themes only as a secondary synthesis after the repo-level inventory is clear

12. Use coverage checks before saving.
     - Review the discovered workstreams and confirm each meaningful cluster is saved, merged into another contribution, or intentionally excluded
     - Present coverage checks as: workstream, disposition, linked candidate, exclusion reason if excluded

13. Optimize for career-material quality.
     - The saved contribution inventory should help the developer prepare for interviews, resumes, promotions, and performance reviews
     - Prefer contributions the developer could plausibly explain and defend in detail

14. Avoid command thrash.
     - If syntax is unclear, use `--help` once and correct course
     - Prefer the live CLI help over copied examples when exact flags or argument order matter
     - Do not repeat the same failing command pattern multiple times

## Summary Checklist

Before marking work as complete, verify:

- [ ] `cargo build` succeeds
- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes (all tests)
- [ ] Missing unit tests have been added
- [ ] All new code has unit tests
- [ ] No regressions in existing functionality
- [ ] If tests failed, maximum 3 attempts were made
- [ ] All changes have been committed with descriptive messages
