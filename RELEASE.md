# Release Process

This document describes the process for creating a new release of Contrack.

## Prerequisites

- Git repository is clean (no uncommitted changes)
- All tests pass: `cargo test`
- Code is linted: `cargo clippy`
- You have push access to the repository

## Release Steps

### 1. Update Version

Update the version in `Cargo.toml`:

```toml
[package]
version = "0.0.1"  # Update this
```

### 2. Update CHANGELOG (Optional but Recommended)

If you maintain a CHANGELOG.md, add an entry for the new version:

```markdown
## [0.0.1] - YYYY-MM-DD

### Added
- Initial release
- Feature X
- Feature Y

### Changed
- Improvement Z

### Fixed
- Bug fix A
```

### 3. Commit Version Changes

```bash
git add Cargo.toml CHANGELOG.md  # if applicable
git commit -m "Bump version to v0.0.1"
```

### 4. Create and Push Git Tag

Create an annotated tag for the release:

```bash
git tag -a v0.0.1 -m "Release v0.0.1"
git push origin v0.0.1
```

Or push all tags:

```bash
git push --tags
```

### 5. GitHub Actions Release

The GitHub Actions workflow (`.github/workflows/release.yml`) will automatically:

1. Detect the new tag (tags matching `v*`)
2. Create a GitHub Release
3. Build binaries for multiple platforms:
   - `x86_64-unknown-linux-gnu` (Linux)
   - `x86_64-pc-windows-msvc` (Windows)
   - `x86_64-apple-darwin` (macOS Intel)
   - `aarch64-apple-darwin` (macOS Apple Silicon)
4. Upload release assets to GitHub Releases

Monitor the workflow at: `https://github.com/isaacadams/contrack/actions`

### 6. Verify Release

1. Check that the GitHub Release was created: `https://github.com/isaacadams/contrack/releases`
2. Verify all platform binaries are attached
3. Test downloading and running a binary

### 7. Publish to crates.io (Optional)

If you want to publish to crates.io:

```bash
# Make sure you're logged in
cargo login

# Dry run to check everything
cargo publish --dry-run

# Publish
cargo publish
```

**Note:** Once published to crates.io, you cannot delete or overwrite a version. Make sure everything is correct before publishing.

## Version Numbering

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR** (1.0.0): Breaking changes
- **MINOR** (0.1.0): New features, backwards compatible
- **PATCH** (0.0.1): Bug fixes, backwards compatible

For pre-1.0 releases, increment MINOR for new features and PATCH for bug fixes.

## Quick Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update CHANGELOG.md (if maintained)
- [ ] Run tests: `cargo test`
- [ ] Run linter: `cargo clippy`
- [ ] Commit version changes
- [ ] Create and push git tag `v0.0.1`
- [ ] Verify GitHub Actions workflow completes successfully
- [ ] Verify GitHub Release was created with all binaries
- [ ] (Optional) Publish to crates.io

## Troubleshooting

### GitHub Actions Workflow Fails

- Check the Actions tab for error details
- Ensure the workflow file `.github/workflows/release.yml` is correct
- Verify you have write permissions to the repository

### Tag Already Exists

If you need to recreate a tag:

```bash
# Delete local tag
git tag -d v0.0.1

# Delete remote tag
git push origin --delete v0.0.1

# Recreate and push
git tag -a v0.0.1 -m "Release v0.0.1"
git push origin v0.0.1
```

### Version Already Published to crates.io

You cannot republish the same version. You must increment the version number.

## Automated Release Script

You can create a simple release script to automate these steps:

```bash
#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: ./release.sh <version>"
    echo "Example: ./release.sh 0.0.1"
    exit 1
fi

# Update version in Cargo.toml
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Commit and tag
git add Cargo.toml
git commit -m "Bump version to v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"

# Push
git push origin main
git push origin "v$VERSION"

echo "Release v$VERSION created! GitHub Actions will build and publish binaries."
```

