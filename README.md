# Contrack

A CLI tool for tracking and documenting code contributions across repositories. Built for AI agents and developers to maintain consistent contribution documentation.

## Features

- 📊 **Track Contributions** - Store and organize code contributions with metadata
- 🔍 **Git Integration** - Automatically extract commit details from git repositories
- 📝 **Markdown Generation** - Generate beautiful contribution documentation
- 🗄️ **SQLite Database** - Store data in a portable SQLite database
- 🤖 **AI-Friendly** - Designed with AI agents in mind, includes built-in rules and prompts
- 🎯 **Cross-Platform** - Works on macOS, Linux, and Windows

## Installation

### Download Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/yourusername/contrack/releases) page:

#### macOS
```bash
# Intel
curl -L https://github.com/yourusername/contrack/releases/latest/download/contrack-x86_64-apple-darwin.tar.gz | tar xz
sudo mv contrack /usr/local/bin/

# Apple Silicon
curl -L https://github.com/yourusername/contrack/releases/latest/download/contrack-aarch64-apple-darwin.tar.gz | tar xz
sudo mv contrack /usr/local/bin/
```

#### Linux
```bash
curl -L https://github.com/yourusername/contrack/releases/latest/download/contrack-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv contrack /usr/local/bin/
```

#### Windows
1. Download `contrack-x86_64-pc-windows-msvc.zip` from the [Releases](https://github.com/yourusername/contrack/releases) page
2. Extract the archive
3. Add the extracted directory to your PATH

### Build from Source

Requires Rust 1.70+ and Cargo.

```bash
git clone https://github.com/yourusername/contrack.git
cd contrack
cargo build --release
sudo cp target/release/contrack /usr/local/bin/
```

## Quick Start

### 1. Initialize a Repository

```bash
contrack init \
  --repo-url "https://github.com/org/repo" \
  --org "MyOrg" \
  --name "my-repo" \
  --description "My awesome repository"
```

### 2. Add a Contribution

```bash
contrack add \
  --repo-url "https://github.com/org/repo" \
  --name "Feature: User Authentication" \
  --overview "Implemented OAuth2 authentication flow" \
  --description "Added comprehensive OAuth2 integration with Google and GitHub providers..." \
  --key-commits "abc123,def456" \
  --category "Core Feature" \
  --priority 9
```

### 3. Update Commit Details

```bash
# From within your git repository
contrack update

# Or specify a path
contrack update --repo-path /path/to/repo
```

### 4. Generate Markdown Documentation

```bash
contrack generate \
  --repo-url "https://github.com/org/repo" \
  --output CONTRIBUTIONS.md

# Filter by author
contrack generate \
  --repo-url "https://github.com/org/repo" \
  --output CONTRIBUTIONS.md \
  --author "John Doe"
```

## Commands

### `init`
Initialize a new repository in the database.

```bash
contrack init --repo-url <URL> --org <ORG> --name <NAME> [--description <DESC>]
```

### `add`
Add a new contribution.

```bash
contrack add \
  --repo-url <URL> \
  --name <NAME> \
  --overview <OVERVIEW> \
  --description <DESC> \
  --key-commits <COMMA_SEPARATED_HASHES> \
  [--related-commits <COMMA_SEPARATED_HASHES>] \
  [--category <CATEGORY>] \
  [--priority <1-10>]
```

### `update`
Extract commit details from git repository and update the database.

```bash
contrack update [--repo-path <PATH>]
```

### `generate`
Generate contributions markdown file.

```bash
contrack generate \
  --repo-url <URL> \
  [--output <FILE>] \
  [--author <AUTHOR>]
```

### `query`
Query the database.

```bash
# List contributions
contrack query contributions --repo-url <URL>

# Show contribution details
contrack query contribution --repo-url <URL> --name <NAME>

# Show commits for a contribution
contrack query commits --repo-url <URL> --name <NAME>

# Show statistics
contrack query stats
```

### `list`
List repositories in the database.

```bash
contrack list [--detailed]
```

## Database Location

The SQLite database is stored in platform-specific application data directories:

- **macOS**: `~/Library/Application Support/com.contrack.contrack/contributions.db`
- **Linux**: `~/.local/share/contrack/contributions.db`
- **Windows**: `%APPDATA%\contrack\contributions.db`

## Usage with AI Agents

Contrack is designed to work seamlessly with AI agents. The database includes:

- **Agent Rules** - Instructions for AI agents on how to use the database
- **Prompts** - Reusable prompt templates for common tasks
- **Structured Data** - Consistent schema for easy querying

### Example AI Workflow

1. **Load the database**:
   ```python
   import sqlite3
   conn = sqlite3.connect('path/to/contributions.db')
   ```

2. **Read agent rules**:
   ```sql
   SELECT * FROM agent_rules ORDER BY priority DESC;
   ```

3. **Query contributions**:
   ```sql
   SELECT * FROM contributions 
   WHERE repository_url = 'https://github.com/org/repo'
   ORDER BY priority DESC;
   ```

4. **Generate markdown** using the `generate_contributions_markdown` prompt

## Categories

Standard contribution categories:

- **Core Feature** - Major features central to the product
- **Integration** - Third-party service integrations
- **Infrastructure** - Backend infrastructure and tooling
- **Feature Enhancement** - Improvements to existing features
- **Feature** - New features (less critical than Core Feature)
- **Configuration** - Configuration changes and state management
- **Performance** - Performance optimizations
- **Bug Fix** - Bug fixes and corrections

## Priority Levels

- **10** - Critical/core features, major architectural changes
- **9-8** - Major features, important integrations
- **7-5** - Important features and enhancements
- **4-1** - Minor features, bug fixes, configuration changes

## Examples

### Complete Workflow

```bash
# 1. Initialize repository
contrack init \
  --repo-url "https://github.com/myorg/myrepo" \
  --org "MyOrg" \
  --name "myrepo"

# 2. Add contributions
contrack add \
  --repo-url "https://github.com/myorg/myrepo" \
  --name "API Authentication" \
  --overview "Implemented JWT-based authentication" \
  --description "Full JWT implementation with refresh tokens..." \
  --key-commits "a1b2c3d,e4f5g6h" \
  --category "Core Feature" \
  --priority 10

# 3. Update commit details from git
cd /path/to/myrepo
contrack update

# 4. Generate documentation
contrack generate \
  --repo-url "https://github.com/myorg/myrepo" \
  --output CONTRIBUTIONS.md
```

### Query Examples

```bash
# List all contributions
contrack query contributions --repo-url "https://github.com/myorg/myrepo"

# Get details for a specific contribution
contrack query contribution \
  --repo-url "https://github.com/myorg/myrepo" \
  --name "API Authentication"

# View commits for a contribution
contrack query commits \
  --repo-url "https://github.com/myorg/myrepo" \
  --name "API Authentication"

# Database statistics
contrack query stats
```

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Running Lints

```bash
cargo clippy
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

Built with ❤️ for developers and AI agents who want to maintain great contribution documentation.

