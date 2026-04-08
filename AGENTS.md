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

1. Use `contrack` as the system of record.
   - Refresh stored evidence with `contrack refresh` or `contrack commit import`
   - Inspect imported evidence with `contrack commit list`
   - Save accepted contribution records with `contrack contribution add`

2. Use `gh` to fetch higher-signal GitHub context.
   - Prefer PR titles, descriptions, authorship, changed files, and commit lists over raw commit messages alone
   - Useful commands include:
     - `gh pr list --author <user> --state all`
     - `gh pr view <number> --json title,body,author,files,commits`
     - `gh search prs --author <user> --repo <owner/repo>`

3. Identify larger contributions, not raw commit logs.
   - Group related commits into one contribution when they clearly serve one outcome
   - Prefer features, major refactors, infrastructure work, reliability work, performance work, and meaningful tooling improvements
   - Avoid creating one contribution per commit unless a single commit fully represents a meaningful standalone accomplishment

4. Group commits using strong evidence.
   - Group by shared PR, feature, issue, subsystem, or tightly related time window
   - Split when one PR contains multiple distinct outcomes or when the work is clearly unrelated

5. Attribute carefully to a specific user.
   - Start with commit author evidence
   - Validate with PR authorship and PR context from `gh`
   - If authorship is mixed or unclear, keep only the clearly attributable portion or mark the grouping as uncertain

6. Be conservative.
   - Fewer, stronger contributions are better than many weak ones
   - Down-rank or ignore formatting-only changes, dependency churn without clear impact, isolated housekeeping, and noisy WIP commits unless they are part of a larger finished outcome

## Summary Checklist

Before marking work as complete, verify:

- [ ] `cargo build` succeeds
- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo test` passes (all tests)
- [ ] Test coverage has been checked with `cargo-tarpaulin`
- [ ] Missing unit tests have been added
- [ ] All new code has unit tests
- [ ] No regressions in existing functionality
- [ ] If tests failed, maximum 3 attempts were made
- [ ] All changes have been committed with descriptive messages
