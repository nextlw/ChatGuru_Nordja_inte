# Contributing to clickup_v2

First off, thank you for considering contributing to clickup_v2! It's people like you that make this crate a great tool for the Rust community.

## Code of Conduct

By participating in this project, you agree to abide by our code of conduct: be respectful, inclusive, and constructive in all interactions.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, please include:

- **Clear title and description**
- **Steps to reproduce**
- **Expected behavior**
- **Actual behavior**
- **Environment details** (OS, Rust version, crate version)
- **Relevant logs** (with `RUST_LOG=debug`)
- **Code samples** if applicable

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- **Use case description**
- **Proposed solution**
- **Alternative solutions considered**
- **Additional context** (mockups, examples, etc.)

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Follow the setup instructions** in the README
3. **Make your changes** following our coding standards
4. **Add tests** for new functionality
5. **Update documentation** as needed
6. **Run the test suite** and ensure all tests pass
7. **Submit a pull request** with a clear description

## Development Setup

### Prerequisites

- Rust 1.70+ (stable)
- cargo and rustup
- A ClickUp account for testing (free tier is fine)

### Setup Steps

```bash
# Clone your fork
git clone https://github.com/yourusername/clickup_v2.git
cd clickup_v2

# Add upstream remote
git remote add upstream https://github.com/originalrepo/clickup_v2.git

# Install development tools
cargo install cargo-watch cargo-edit cargo-audit

# Copy environment template
cp .env.example .env

# Edit .env with your ClickUp OAuth credentials
# Get credentials from: https://app.clickup.com/settings/integrations

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) with these additions:

- **Use `rustfmt`**: Run `cargo fmt` before committing
- **Use `clippy`**: Run `cargo clippy` and fix all warnings
- **Documentation**: All public APIs must have doc comments
- **Examples**: Include examples in doc comments where helpful
- **Error messages**: Should be helpful and actionable

### Code Organization

```rust
// Good: Clear module organization
pub mod auth {
    pub mod oauth;
    mod callback;  // Private submodule
    mod token;
}

// Good: Clear, documented public API
/// Authenticates with ClickUp and returns an access token.
///
/// # Examples
/// ```
/// let oauth = OAuthFlow::new()?;
/// let token = oauth.authenticate().await?;
/// ```
///
/// # Errors
/// Returns `AuthError` if authentication fails.
pub async fn authenticate(&self) -> AuthResult<String> {
    // Implementation
}
```

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
feat: add support for task attachments
fix: resolve token expiration edge case
docs: update OAuth2 flow documentation
test: add integration tests for custom fields
refactor: simplify cache implementation
perf: optimize entity search performance
chore: update dependencies
```

### Testing

#### Unit Tests
Place unit tests in the same file as the code:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_validation() {
        // Test implementation
    }
}
```

#### Integration Tests
Place integration tests in the `tests/` directory:

```rust
// tests/oauth_flow.rs
#[tokio::test]
async fn test_complete_oauth_flow() {
    // Test implementation
}
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_token_validation

# Run tests with output
cargo test -- --nocapture

# Run only integration tests
cargo test --test '*'

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Documentation

#### Code Documentation

```rust
/// Creates a new task with custom fields.
///
/// # Arguments
///
/// * `list_id` - The ID of the list to create the task in
/// * `request` - The task creation request with all fields
///
/// # Returns
///
/// Returns the created task response on success.
///
/// # Examples
///
/// ```
/// let task = client.create_task(
///     "list_123",
///     CreateTaskRequest::builder()
///         .name("New Task")
///         .build()
/// ).await?;
/// ```
///
/// # Errors
///
/// - `AuthError::TokenError` - If authentication fails
/// - `AuthError::ApiError` - If the API request fails
pub async fn create_task(
    &self,
    list_id: &str,
    request: CreateTaskRequest
) -> AuthResult<TaskResponse> {
    // Implementation
}
```

#### README Updates

Update the README when adding:
- New features
- Breaking changes
- New dependencies
- New environment variables

## Review Process

### What We Look For

1. **Code Quality**
   - Clean, readable code
   - Appropriate abstractions
   - No unnecessary complexity

2. **Testing**
   - Adequate test coverage
   - Tests pass consistently
   - Edge cases handled

3. **Documentation**
   - Clear API documentation
   - Updated README if needed
   - Helpful commit messages

4. **Performance**
   - No performance regressions
   - Efficient algorithms
   - Appropriate use of async

5. **Security**
   - No credential leaks
   - Secure token handling
   - Input validation

### Review Timeline

- Initial review: Within 48 hours
- Follow-up reviews: Within 24 hours
- Small fixes: Often same day

## Release Process

1. **Version Bump**: Update version in `Cargo.toml`
2. **Changelog**: Update `CHANGELOG.md`
3. **Documentation**: Ensure docs are current
4. **Testing**: All tests must pass
5. **PR Merge**: Merge to main branch
6. **Automatic Release**: CI/CD handles the rest

## Getting Help

- **Discord**: Join our Discord server (link in README)
- **Issues**: Open a GitHub issue with the `question` label
- **Discussions**: Use GitHub Discussions for general topics

## Recognition

Contributors are recognized in:
- The project README
- Release notes
- Annual contributor spotlight (for regular contributors)

## License

By contributing, you agree that your contributions will be licensed under the same MIT License that covers the project.

---

Thank you for contributing to clickup_v2! Your efforts help make this crate better for everyone in the Rust community.