# Cargo.toml Configuration Documentation

## Overview
This is the main Cargo configuration file for the Rust Blog CMS Rust backend. It defines project metadata, dependencies, build configurations, and multiple binary targets for the high-performance API server.

## File Purpose
- Configures the Rust project structure and metadata
- Manages all external dependencies and their versions
- Defines multiple binary targets for different utilities
- Sets build features and compiler options
- Ensures reproducible builds with pinned dependency versions

## Package Configuration

### Basic Information
```toml
[package]
name = "linux-tutorial-backend"
version = "1.0.0"
edition = "2021"
rust-version = "1.82"
authors = ["zerox80"]
license = "MIT"
```

#### Fields Explained
- **name**: Crate name used in Rust ecosystem and package registry
- **version**: Semantic version following SemVer conventions
- **edition**: Rust edition (2021 enables modern Rust features)
- **rust-version**: Minimum supported Rust version for compatibility
- **authors**: Project author(s) for attribution
- **license**: Software license type (MIT for permissive licensing)

### Project Metadata
```toml
description = "High-performance Rust backend API for Rust Blog CMS"
repository = "https://github.com/zerox80/RustBlogCMS"
keywords = ["cms", "api", "tutorial", "axum", "education"]
categories = ["web-programming", "web-programming::http-server"]
```

#### Metadata Fields
- **description**: Brief project description for crate registry
- **repository**: Source code repository URL
- **keywords**: Search terms for crate discovery
- **categories**: Official crate.io categories for classification

## Dependencies Configuration

### Core Web Framework
```toml
axum = "0.8"
axum-extra = { version = "0.12", features = ["typed-header", "cookie"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "limit"] }
```

#### Web Stack Components
- **axum (0.8)**: Modern async web framework built on Tokio
- **axum-extra**: Additional utilities for typed headers and cookies
- **tokio**: Async runtime with full feature set
- **tower**: Middleware and service abstractions
- **tower-http**: HTTP-specific middleware (CORS, request limits)

### Security and Authentication
```toml
tower_governor = "0.8"
jsonwebtoken = "9.3"
bcrypt = "0.17"
sha2 = "0.10"
hmac = "0.12"
subtle = "2.5"
```

#### Security Dependencies
- **tower_governor**: Rate limiting middleware
- **jsonwebtoken**: JWT token creation and validation
- **bcrypt**: Password hashing for secure authentication
- **sha2**: SHA-2 cryptographic hash functions
- **hmac**: HMAC-based message authentication
- **subtle**: Constant-time cryptographic operations

### Database and Serialization
```toml
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "sqlite"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

#### Data Management
- **sqlx**: Async SQL toolkit with SQLite support
- **serde**: Serialization framework with derive macros
- **serde_json**: JSON serialization/deserialization

### Utilities and Standard Library Extensions
```toml
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
uuid = { version = "1.6", features = ["v4", "serde"] }
url = "2.5"
anyhow = "1.0"
time = "0.3"
```

#### Utility Libraries
- **chrono**: Date and time handling with serde integration
- **dotenv**: Environment variable loading from .env files
- **tracing**: Structured logging and diagnostics
- **tracing-subscriber**: Log subscriber with filtering
- **uuid**: UUID generation with v4 and serde support
- **url**: URL parsing and manipulation
- **anyhow**: Error handling with context
- **time**: Additional time utilities

### Security-Focused Dependencies
```toml
idna_adapter = "=1.2.1"
regex = { version = "1.10", default-features = false, features = ["std"] }
unicode-normalization = "0.1"
```

#### Security and Validation
- **idna_adapter**: Internationalized domain names (pinned version)
- **regex**: Regular expressions with minimal features for security
- **unicode-normalization**: Unicode text normalization

### Pinned Dependencies (Security)
```toml
[dependencies.home]
version = "=0.5.9"

[dependencies.base64ct]
version = "=1.6.0"
default-features = false
features = ["alloc"]
```

#### Security Considerations
- **home**: Pinned version to prevent dependency confusion
- **base64ct**: Constant-time base64 encoding with minimal features

## Binary Targets Configuration

### Main API Server
The main binary is implicitly defined at `src/main.rs` and serves as the primary API server.

### Content Export Utility
```toml
[[bin]]
name = "export_content"
path = "src/bin/export_content.rs"
```

#### Export Functionality
- **Purpose**: Export tutorial content from database to files
- **Usage**: `cargo run --bin export_content`
- **Features**: Batch export, format conversion, backup creation

### Content Import Utility
```toml
[[bin]]
name = "import_content"
path = "src/bin/import_content.rs"
```

#### Import Functionality
- **Purpose**: Import tutorial content from files to database
- **Usage**: `cargo run --bin import_content`
- **Features**: Batch import, validation, duplicate handling

## Security Considerations

### Dependency Security
- Pinned versions for security-critical dependencies
- Minimal feature sets to reduce attack surface
- Regular security updates through `cargo audit`
- Use of `cargo-deny` for policy enforcement

### Runtime Security
- Rate limiting with tower_governor
- Secure password hashing with bcrypt
- JWT-based authentication with proper validation
- SQL injection prevention through sqlx parameterized queries

## Performance Considerations

### Async Runtime
- Tokio with full feature set for optimal performance
- Async/await throughout the codebase
- Efficient connection pooling and resource management

### Database Performance
- SQLite for lightweight deployment with async support
- Connection pooling through sqlx
- Prepared statements and query optimization

### Memory Management
- Rust's ownership system prevents memory leaks
- Efficient string handling with Cow types where appropriate
- Minimal runtime overhead through zero-cost abstractions

## Development and Build Configuration

### Build Features
Consider adding conditional features for different environments:

```toml
[features]
default = ["sqlite"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
dev = ["tracing-subscriber/std", "tracing-subscriber/ansi"]
```

### Development Dependencies
Consider adding for development and testing:

```toml
[dev-dependencies]
tokio-test = "0.4"
criterion = "0.5"
proptest = "1.0"
```

## Environment Configuration

### Required Environment Variables
- `DATABASE_URL`: SQLite database connection string
- `JWT_SECRET`: Secret key for JWT token signing
- `RUST_LOG`: Logging level configuration
- `SERVER_PORT`: Server listening port (default: 8489)

### Optional Environment Variables
- `CORS_ORIGINS`: Allowed CORS origins
- `RATE_LIMIT`: Rate limiting configuration
- `LOG_FORMAT`: Log output format

## Usage Instructions

### Development Setup
```bash
# Install dependencies
cargo build

# Run development server
cargo run

# Run with specific log level
RUST_LOG=debug cargo run

# Run content export
cargo run --bin export_content

# Run content import
cargo run --bin import_content
```

### Production Build
```bash
# Optimized build
cargo build --release

# Run production server
./target/release/linux-tutorial-backend

# Run with environment file
dotenv -f .env.production ./target/release/linux-tutorial-backend
```

## Testing and Quality Assurance

### Unit Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Linting and Formatting
```bash
# Run Clippy lints
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without changes
cargo fmt --check
```

### Security Auditing
```bash
# Audit dependencies for vulnerabilities
cargo audit

# Check outdated dependencies
cargo outdated

# Enforce dependency policies
cargo deny check
```

## Performance Monitoring

### Profiling
```bash
# Install profiling tools
cargo install cargo-flamegraph

# Generate flame graph
cargo flamegraph --bin linux-tutorial-backend

# CPU profiling
perf record --call-graph dwarf ./target/release/linux-tutorial-backend
```

### Benchmarking
```bash
# Run benchmarks
cargo bench

# Compare benchmark results
cargo bench --bench benchmark_name
```

## Deployment Considerations

### Container Optimization
- Use multi-stage Docker builds
- Minimize runtime dependencies
- Optimize binary size with cargo-strip
- Consider using `cargo-chef` for faster Docker builds

### Production Configuration
- Set appropriate log levels (warn/error)
- Configure connection pooling
- Enable rate limiting
- Set up proper CORS policies
- Implement health checks

## Maintenance and Updates

### Dependency Updates
```bash
# Update dependencies
cargo update

# Update specific dependency
cargo update -p package_name

# Check for security updates
cargo audit
```

### Version Management
- Follow semantic versioning
- Maintain CHANGELOG.md
- Use git tags for releases
- Automate releases with GitHub Actions

## Troubleshooting

### Common Issues
- **Compilation errors**: Check Rust version compatibility
- **Runtime errors**: Verify environment variables
- **Database connection issues**: Check DATABASE_URL format
- **Performance issues**: Review query patterns and indexing

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Enable SQL query logging
RUST_LOG=sqlx=debug cargo run

# Enable request tracing
RUST_LOG=tower_http=debug cargo run
```

## Integration with Frontend

### API Documentation
- OpenAPI/Swagger documentation at `/api/docs`
- Interactive API testing interface
- Request/response schema documentation

### CORS Configuration
- Configure allowed origins for development
- Set proper headers for production
- Handle preflight requests appropriately