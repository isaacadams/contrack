# Development Rules and Guidelines for AI Agents

This document provides instructions for AI agents working on the contrack codebase. Follow these guidelines to ensure code quality and maintainability.

## Pre-Commit Verification

Before completing any changes, you MUST run the following commands to verify your work:

1. **Code Quality Check**
   ```bash
   cargo clippy
   ```
   - Fix all clippy warnings and errors
   - Ensure code follows Rust best practices

2. **Compilation Verification**
   ```bash
   cargo build
   cargo build --release
   ```
   - Both debug and release builds must succeed
   - Fix any compilation errors before proceeding

3. **Test Suite**
   ```bash
   cargo test
   ```
   - All existing tests must pass
   - No regressions in existing functionality
   - If tests fail, see "Test Failure Protocol" below

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

## Summary Checklist

Before marking work as complete, verify:

- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo build --release` succeeds
- [ ] `cargo test` passes (all tests)
- [ ] Test coverage has been checked with `cargo-tarpaulin`
- [ ] Missing unit tests have been added
- [ ] All new code has unit tests
- [ ] No regressions in existing functionality
- [ ] If tests failed, maximum 3 attempts were made

