# Contrack - Project Summary

## Overview

**Contrack** is a Rust CLI tool designed for tracking and documenting code contributions across repositories. It's purpose-built to be used by AI agents and developers to maintain consistent contribution documentation.

## Project Structure

```
contrack/
├── Cargo.toml              # Rust project configuration
├── README.md               # User-facing documentation
├── LICENSE-MIT             # MIT License
├── LICENSE-APACHE          # Apache 2.0 License
├── .gitignore             # Git ignore rules
├── .github/
│   └── workflows/
│       └── release.yml     # GitHub Actions release workflow
└── src/
    ├── main.rs             # CLI entry point and command parsing
    ├── database.rs         # SQLite database operations
    ├── commands.rs         # Command implementations
    ├── git.rs              # Git repository integration
    ├── markdown.rs         # Markdown generation
    └── utils.rs            # Utility functions (database path, etc.)
```

## Key Features

1. **Repository Management** - Initialize and track repositories
2. **Contribution Tracking** - Add, update, and query contributions
3. **Git Integration** - Extract commit details from git repositories
4. **Markdown Generation** - Generate contribution documentation
5. **AI-Friendly** - Built-in agent rules and prompts
6. **Cross-Platform** - Works on macOS, Linux, and Windows

## Database Schema

The tool uses SQLite with the following tables:

- `repositories` - Repository metadata
- `contributions` - Contribution records with JSON fields
- `commits` - Detailed commit information
- `agent_rules` - Instructions for AI agents
- `prompts` - Reusable prompt templates

## Database Location

Uses platform-specific application data directories via the `directories` crate:

- **macOS**: `~/Library/Application Support/com.contrack.contrack/`
- **Linux**: `~/.local/share/contrack/`
- **Windows**: `%APPDATA%\contrack\`

## Commands

- `init` - Initialize a repository
- `add` - Add a contribution
- `update` - Update commit details from git
- `generate` - Generate markdown documentation
- `query` - Query the database (contributions, commits, stats)
- `list` - List repositories

## Dependencies

- `clap` - CLI argument parsing
- `rusqlite` - SQLite database
- `serde` / `serde_json` - JSON serialization
- `directories` - Platform-specific directories
- `git2` - Git repository access
- `chrono` - Date/time handling
- `colored` - Terminal colors
- `anyhow` - Error handling

## Build & Release

The project includes a GitHub Actions workflow that:

1. Triggers on version tags (v*)
2. Builds for multiple platforms:
   - x86_64-unknown-linux-gnu
   - x86_64-pc-windows-msvc
   - x86_64-apple-darwin
   - aarch64-apple-darwin
3. Creates release archives (tar.gz for Unix, zip for Windows)
4. Uploads assets to GitHub Releases

## Usage Example

```bash
# Initialize
contrack init --repo-url "https://github.com/org/repo" --org "Org" --name "repo"

# Add contribution
contrack add --repo-url "..." --name "Feature X" --overview "..." --description "..." --key-commits "abc,def"

# Update from git
contrack update

# Generate markdown
contrack generate --repo-url "..." --output CONTRIBUTIONS.md
```

## Next Steps

1. Update repository URLs in README.md and Cargo.toml
2. Test the tool with real repositories
3. Create initial release tag to trigger GitHub Actions
4. Customize agent rules and prompts as needed
5. Add additional features based on usage

## Notes

- The tool is generic and doesn't reference any specific organizations or people
- All data is stored locally in SQLite
- Designed to be easily copied to a separate repository
- AI agents can read the database directly or use the CLI tool

