# ==============================================================================
# RustBlogCMS Backend Dockerfile
# ==============================================================================
#
# Multi-stage Docker build for the Rust Blog CMS Rust backend service.
# This Dockerfile creates an optimized, secure production container for the
# Rust-based API server.
#
# Architecture:
# Stage 1: Rust builder - Compiles optimized Rust binary
# Stage 2: Runtime container - Minimal Debian with runtime dependencies
#
# Features:
# - Multi-stage build for minimal final image size
# - Optimized Rust compilation with dependency caching
# - Security-hardened runtime environment
# - Health check integration for container orchestration
# - Reproducible builds with Cargo.lock
#
# Build Arguments:
# - CARGO_BUILD_FLAGS: Additional cargo build flags (optional)
# - RUSTFLAGS: Rust compiler flags (optional)
#
# Environment Variables:
# - PORT: Server listening port (default: 8489)
# - RUST_LOG: Log level configuration (trace,debug,info,warn,error)
# - DATABASE_URL: Database connection string
# - JWT_SECRET: JWT token secret key
#
# @version 1.0.0
# @author RustBlogCMS Team
# ==============================================================================

# ==============================================================================
# STAGE 1: RUST BUILD ENVIRONMENT
# ==============================================================================

# Use latest stable Rust for building
# Provides access to latest Rust features and optimizations
FROM rustlang/rust:nightly-bookworm AS builder

# Set working directory for all subsequent commands
WORKDIR /app

# ==============================================================================
# BUILD DEPENDENCIES
# ==============================================================================

# Install system dependencies required for compilation
# pkg-config: For system library discovery
# libssl-dev: OpenSSL development headers for TLS/HTTPS support
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# ==============================================================================
# DEPENDENCY CACHING STRATEGY
# ==============================================================================

# Copy Cargo configuration files first to leverage Docker layer caching
# This ensures dependencies are only rebuilt when Cargo.toml changes
# Improves build performance significantly in CI/CD pipelines
COPY Cargo.toml Cargo.lock* ./

# Ensure reproducible builds with Cargo.lock
# Generate Cargo.lock if it doesn't exist for dependency pinning
# This ensures consistent dependency versions across builds
RUN if [ ! -f Cargo.lock ]; then \
        echo "INFO: Cargo.lock not found, generating..."; \
        cargo generate-lockfile; \
    fi

# ==============================================================================
# DEPENDENCY PRE-COMPILATION
# ==============================================================================

# Create dummy source files to pre-build dependencies
# This creates a cached Docker layer with all dependencies compiled
# Subsequent builds will only recompile the actual source code
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/bin/export_content.rs && \
    echo "fn main() {}" > src/bin/import_content.rs && \
    cargo build --release && \
    rm -rf src

# ==============================================================================
# APPLICATION COMPILATION
# ==============================================================================

# Copy all source code including modules, tests, and assets
# This includes the actual application logic and configuration
COPY . .

# Build the release version of the application
# --release enables optimizations for production performance
# touch ensures proper file timestamps for incremental builds
RUN touch src/main.rs && \
    cargo build --release

# Verify binary was built successfully
RUN test -f target/release/rust-blog-backend || (echo "Build failed - binary not found" && exit 1)

# ==============================================================================
# STAGE 2: PRODUCTION RUNTIME CONTAINER
# ==============================================================================

# Use Debian Bookworm Slim for minimal runtime footprint
# Provides essential libraries while maintaining security and stability
# Slim variant reduces attack surface and image size
FROM debian:bookworm-slim

# Set working directory for application execution
WORKDIR /app

# ==============================================================================
# RUNTIME DEPENDENCIES
# ==============================================================================

# Install minimal runtime dependencies
# ca-certificates: For HTTPS/TLS certificate validation
# libssl3: OpenSSL runtime library for secure connections
# curl: For health checks and API testing
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# ==============================================================================
# APPLICATION DEPLOYMENT
# ==============================================================================

# Copy compiled binary from builder stage
# Only the optimized binary, not source code or build artifacts
COPY --from=builder /app/target/release/rust-blog-backend /app/backend

# Set appropriate permissions for the binary
# Ensure the binary is executable and has correct ownership
RUN chmod +x /app/backend

# ==============================================================================
# SECURITY CONFIGURATION
# ==============================================================================

# Create non-root user for improved security
# This follows security best practices for containerized applications
RUN useradd -m -u 1000 -s /bin/bash appuser && \
    chown -R appuser:appuser /app && \
    mkdir -p /data && \
    chown -R appuser:appuser /data
USER appuser

# ==============================================================================
# NETWORK CONFIGURATION
# ==============================================================================

# Expose the API service port
# This port should match the configuration in the application
EXPOSE 8489

# ==============================================================================
# HEALTH MONITORING
# ==============================================================================

# Configure health check for container orchestration systems
# Enables automatic restart, load balancing, and monitoring integration
# Checks application health every 30 seconds with proper timeouts
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8489/api/health || exit 1

# ==============================================================================
# CONTAINER EXECUTION
# ==============================================================================

# Execute the Rust backend application
# This is the main entry point for the container
CMD ["/app/backend"]

# ==============================================================================
# BUILD INSTRUCTIONS
# ==============================================================================
#
# Build the backend container:
# docker build -t rust-blog-cms-backend -f backend/Dockerfile backend/
#
# Run the container with environment variables:
# docker run -p 8489:8489 \
#   -e RUST_LOG=info \
#   -e DATABASE_URL=postgresql://user:pass@localhost/db \
#   -e JWT_SECRET=your-secret-key \
#   rust-blog-cms-backend
#
# With volume mounts for configuration:
# docker run -p 8489:8489 \
#   -v $(pwd)/config.toml:/app/config.toml \
#   -e RUST_LOG=debug \
#   rust-blog-cms-backend
#
# With custom build arguments:
# docker build \
#   --build-arg RUSTFLAGS="-C target-cpu=native" \
#   -t rust-blog-cms-backend \
#   -f backend/Dockerfile backend/
#

# ==============================================================================
# PRODUCTION DEPLOYMENT GUIDELINES
# ==============================================================================
#
# Security Best Practices:
# 1. Use secrets management for sensitive configuration
# 2. Implement network policies and firewall rules
# 3. Regular security updates and vulnerability scanning
# 4. Enable SELinux/AppArmor profiles
# 5. Use read-only filesystem where possible
# 6. Implement rate limiting and API authentication
#
# Performance Optimization:
# 1. Tune Rust compiler flags for target architecture
# 2. Configure appropriate connection pooling
# 3. Enable compression for API responses
# 4. Implement caching strategies for frequently accessed data
# 5. Monitor memory usage and implement limits
# 6. Use connection keep-alive and HTTP/2
#
# Monitoring and Observability:
# 1. Implement structured logging with correlation IDs
# 2. Configure metrics collection (Prometheus, etc.)
# 3. Set up distributed tracing
# 4. Monitor application performance and error rates
# 5. Implement alerting for critical failures
# 6. Use APM tools for deep performance insights
#
# CI/CD Integration:
# 1. Implement automated testing in build pipeline
# 2. Use semantic versioning for releases
# 3. Implement canary or blue-green deployments
# 4. Configure automated rollback mechanisms
# 5. Use multi-architecture builds (amd64, arm64)
# 6. Implement security scanning in CI/CD pipeline
#
# Scaling Considerations:
# 1. Design for horizontal scalability
# 2. Implement stateless application design
# 3. Use external session storage (Redis, etc.)
# 4. Configure load balancer health checks
# 5. Implement circuit breakers for external services
# 6. Plan database scaling and sharding strategies
#
