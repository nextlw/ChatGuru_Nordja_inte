# clickup_v2

[![Crates.io](https://img.shields.io/crates/v/clickup_v2.svg)](https://crates.io/crates/clickup_v2)
[![Documentation](https://img.shields.io/docsrs/clickup_v2)](https://docs.rs/clickup_v2)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/clickup_v2)](https://crates.io/crates/clickup_v2)

A comprehensive Rust client library and CLI for the ClickUp API v2, featuring OAuth2 authentication, full task management with custom fields support, and intelligent caching.

## Features

- **Complete OAuth2 Flow**: Secure authentication with automatic token management
- **Multi-Environment Support**: Seamlessly works in development (local) and production (cloud) environments
- **Task Management**: Create, read, update, and delete tasks with full custom field support
- **Entity Search**: Search spaces, folders, lists, and tasks with intelligent caching (3-hour TTL)
- **CLI Tool**: Powerful command-line interface for all operations
- **Type Safety**: Fully typed API with comprehensive error handling
- **Async/Await**: Modern async Rust with Tokio runtime

## Installation

### As a Library

Add to your `Cargo.toml`:

```toml
[dependencies]
clickup_v2 = "0.1.1"
```

### As a CLI Tool

Install directly from crates.io:

```bash
cargo install clickup_v2
```

Or build from source:

```bash
# Using cargo install directly from git
cargo install --git https://github.com/nextlw/crate_clickup_v2.git

# Or clone and build locally
git clone https://github.com/nextlw/crate_clickup_v2.git
cd crate_clickup_v2
cargo install --path .
```

## Quick Start

### Library Usage

```rust
use clickup_v2::{ClickUpClient, auth::OAuthFlow};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Authenticate and get access token
    let oauth = OAuthFlow::new()?;
    let token = oauth.authenticate().await?;

    // Create client
    let client = ClickUpClient::new(token, None);

    // Get user info
    let user = client.get_authorized_user().await?;
    println!("Authenticated as: {}", user["username"]);

    // Create a task with custom fields
    let task = client.create_task(
        "list_id",
        CreateTaskRequest::builder()
            .name("New Feature")
            .content("Implement OAuth2 flow")
            .priority(2)
            .custom_field("field_id", CustomFieldValue::Text("In Progress".to_string()))
            .build()
    ).await?;

    Ok(())
}
```

### CLI Usage

```bash
# Authenticate (first time setup)
clickup_v2 login

# Get user info
clickup_v2 user

# List all workspaces
clickup_v2 list-teams

# Create a task
clickup_v2 create-task \
  --list-id "123456" \
  --name "New Feature" \
  --content "Implement new functionality" \
  --priority 2 \
  --custom-field "field_123:text:In Progress"

# Search for entities
clickup_v2 search --name "My Project" --type folder

# Get tasks from a list
clickup_v2 get-tasks --list-id "123456"
```

## Environment Setup

### Development Environment

Create a `.env` file in your project root:

```env
# OAuth2 Credentials (required)
CLICKUP_CLIENT_ID=your_client_id
CLICKUP_CLIENT_SECRET=your_client_secret
CLICKUP_REDIRECT_URI=http://localhost:8888/callback

# Optional
CLICKUP_TEAM_ID=your_default_team_id
CLICKUP_API_URL=https://api.clickup.com/api/v2  # Custom API URL if needed

# Auto-populated after authentication
CLICKUP_ACCESS_TOKEN=your_access_token
```

The crate will automatically generate a template `.env` file if missing when you run it.

### Production Environment

Set environment variables directly:

```bash
export CLICKUP_CLIENT_ID=your_client_id
export CLICKUP_CLIENT_SECRET=your_client_secret
export CLICKUP_REDIRECT_URI=https://your-app.com/callback
export CLICKUP_ACCESS_TOKEN=your_token  # Pre-authorized token
```

The crate automatically detects cloud environments (Google Cloud, AWS, Azure) and adjusts behavior accordingly.

## Custom Fields Support

The library supports all ClickUp custom field types:

```rust
use clickup_v2::CustomFieldValue;

// Text field
CustomFieldValue::Text("In Progress".to_string())

// Number field
CustomFieldValue::Number(42.5)

// Date field (Unix timestamp in milliseconds)
CustomFieldValue::Date(1704067200000)

// Dropdown
CustomFieldValue::DropdownOption("option_id".to_string())

// Multiple options
CustomFieldValue::DropdownOptions(vec!["id1".to_string(), "id2".to_string()])

// Checkbox
CustomFieldValue::Checkbox(true)

// Rating (1-5)
CustomFieldValue::Rating(4)

// And more: Email, Phone, URL, Location, etc.
```

## API Coverage

### Authentication

- [X] OAuth2 flow with PKCE
- [X] Token management and refresh
- [X] Multi-environment support

### User & Teams

- [X] Get authorized user
- [X] Get authorized teams/workspaces

### Hierarchy Navigation

- [X] Get/Search spaces
- [X] Get/Search folders
- [X] Get/Search lists
- [X] Get/Search tasks

### Task Management

- [X] Create tasks with custom fields
- [X] Update tasks
- [X] Delete tasks
- [X] Get task details

### Caching

- [X] 3-hour TTL cache for entity searches
- [X] Cache invalidation
- [X] Cache statistics

## CI/CD Pipeline

This project uses GitHub Actions for continuous integration and deployment:

### Automated Testing

- **Multi-OS Testing**: Ubuntu, macOS, Windows
- **Rust Versions**: Stable and Beta channels
- **Security Audits**: Automatic vulnerability scanning
- **Code Quality**: Rustfmt and Clippy checks

### Automatic Publication

The crate is automatically published to crates.io when:

1. Version is bumped in `Cargo.toml`
2. Changes are pushed to the main branch
3. All tests pass

### Manual Release

For manual releases:

1. Update version in `Cargo.toml`
2. Push to main branch
3. CI/CD will automatically:
   - Run tests
   - Create git tag
   - Publish to crates.io
   - Create GitHub release

### Setting up CI/CD

1. Add these secrets to your GitHub repository:

   - `CRATES_IO_TOKEN`: Your crates.io API token
   - `CLICKUP_CLIENT_ID`: For running tests (optional)
   - `CLICKUP_CLIENT_SECRET`: For running tests (optional)
   - `CLICKUP_ACCESS_TOKEN`: For running tests (optional)
2. The workflows will automatically run on:

   - Push to main/master/develop branches
   - Pull requests
   - Manual workflow dispatch

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/nextlw/crate_clickup_v2.git
cd crate_clickup_v2

# Build the project
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_task

# Run integration tests (requires .env)
cargo test --features integration-tests
```

### Debugging OAuth Flow

Enable debug logging to troubleshoot authentication:

```bash
RUST_LOG=debug cargo run
```

Key log markers:

- `🔑 Starting authentication` - Auth flow begins
- `🌐 Authorization URL generated` - OAuth URL created
- `⏳ Waiting for authorization` - Callback server ready
- `✅ Authorization code received` - Callback successful
- `🔄 Exchanging code for token` - Token exchange

## Architecture

### Module Structure

```
src/
├── auth/
│   ├── oauth.rs      # OAuth2 flow orchestration
│   ├── callback.rs   # Local callback server
│   └── token.rs      # Token management
├── client/
│   └── api.rs        # ClickUp API client
├── config/
│   └── env.rs        # Environment configuration
├── error/
│   └── auth_error.rs # Error types
├── lib.rs            # Library interface
└── main.rs           # CLI implementation
```

### Key Design Patterns

- **Builder Pattern**: Request construction
- **Type State Pattern**: OAuth flow states
- **Repository Pattern**: Token persistence
- **Strategy Pattern**: Environment detection

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `cargo fmt` and `cargo clippy`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/nextlw/crate_clickup_v2/issues)
- **Discussions**: [GitHub Discussions](https://github.com/nextlw/crate_clickup_v2/discussions)
- **Documentation**: [docs.rs](https://docs.rs/clickup_v2)

## Acknowledgments

Built with these amazing Rust crates:

- [reqwest](https://crates.io/crates/reqwest) - HTTP client
- [tokio](https://crates.io/crates/tokio) - Async runtime
- [oauth2](https://crates.io/crates/oauth2) - OAuth2 implementation
- [clap](https://crates.io/crates/clap) - CLI parsing
- [serde](https://crates.io/crates/serde) - Serialization

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
