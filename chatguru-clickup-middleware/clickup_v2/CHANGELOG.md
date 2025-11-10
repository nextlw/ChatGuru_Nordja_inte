# Changelog

All notable changes to the clickup_v2 project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2024-11-10

### Fixed
- Updated README badges to use more reliable shields.io formats
- Fixed documentation badge to use docsrs format
- Fixed license badge to use static MIT badge
- Corrected GitHub repository links to point to https://github.com/nextlw/crate_clickup_v2

### Changed
- Updated all repository references in documentation
- Improved badge compatibility for newly published crates

## [0.1.0] - 2024-11-10

### Added
- Initial release of clickup_v2 crate
- Complete OAuth2 authentication flow with PKCE support
- Multi-environment support (development and production)
- Full task management with custom field support
- Entity search with intelligent caching (3-hour TTL)
- CLI tool with 16+ commands
- Support for all ClickUp custom field types:
  - Text, Number, Date, Checkbox
  - Single and multiple dropdown options
  - Rating, Email, Phone, URL
  - Currency, Location, Progress
- Comprehensive error handling with typed errors
- Async/await support with Tokio runtime
- Token persistence and automatic refresh
- Workspace/Team management
- Space, Folder, List, and Task operations
- API request logging and debugging support

### Security
- CSRF protection with state tokens
- Secure token storage
- HTTPS-only communication
- Redirect URI whitelist validation

## [0.0.1] - 2024-11-09

### Added
- Project initialization
- Basic repository structure
- Initial OAuth2 implementation draft

---

## Version Guidelines

### Version Numbering
We use Semantic Versioning (MAJOR.MINOR.PATCH):
- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible new functionality
- **PATCH**: Backwards-compatible bug fixes

### Pre-release Versions
Pre-release versions are denoted with suffixes:
- `-alpha.X`: Early development, unstable API
- `-beta.X`: Feature complete, testing phase
- `-rc.X`: Release candidate, final testing

### Examples
- `1.0.0`: First stable release
- `1.1.0`: New features added
- `1.1.1`: Bug fixes
- `2.0.0`: Breaking changes
- `1.2.0-beta.1`: Beta release for version 1.2.0