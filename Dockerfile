# ==============================================================================
# RustBlogCMS Frontend Dockerfile
# ==============================================================================
#
# Multi-stage Docker build for the Rust Blog CMS frontend application.
# This Dockerfile creates an optimized production container using Nginx
# to serve the React/Vite application.
#
# Architecture:
# Stage 1: Node.js builder - Compiles React/Vite application
# Stage 2: Nginx production - Serves static files efficiently
#
# Features:
# - Multi-stage build for smaller final image
# - Alpine Linux for minimal attack surface
# - Optimized static file serving
# - Production-ready Nginx configuration
# - Security hardening considerations
#
# Build Arguments:
# - NODE_ENV: Set to 'production' for optimized builds
# - VITE_API_URL: Backend API endpoint (optional)
#
# Environment Variables:
# - PORT: Nginx listening port (default: 80)
#
# @version 1.0.0
# @author RustBlogCMS Team
# ==============================================================================

# ==============================================================================
# STAGE 1: FRONTEND BUILD ENVIRONMENT
# ==============================================================================

# Use Node.js 25 Alpine for smaller image size and security
# Alpine Linux reduces attack surface and image size significantly
FROM node:25-alpine as builder

# Set working directory for all subsequent commands
WORKDIR /app

# ==============================================================================
# DEPENDENCY MANAGEMENT
# ==============================================================================

# Install Node.js dependencies with caching optimization
# Copy package files first to leverage Docker layer caching
# Dependencies will only be reinstalled when package.json changes
COPY package*.json ./

# Install dependencies with legacy peer deps for compatibility
# Consider using npm ci for faster, more reliable builds in CI/CD
# --legacy-peer-deps handles potential peer dependency conflicts
RUN npm install --legacy-peer-deps

# ==============================================================================
# APPLICATION BUILD
# ==============================================================================

# Copy entire application source code
# This includes React components, assets, configuration files, etc.
COPY . .

# Build the production application using Vite
# Vite creates optimized static assets in the dist/ directory
# Production build includes minification, tree shaking, and optimization
RUN npm run build

# Verify build output exists
RUN test -d dist || (echo "Build failed - dist directory not found" && exit 1)

# ==============================================================================
# STAGE 2: PRODUCTION WEB SERVER
# ==============================================================================

# Use Nginx Alpine for production serving
# Nginx provides high-performance static file serving
# Alpine variant maintains minimal image size and security
FROM nginx:alpine

# ==============================================================================
# CONFIGURATION AND DEPLOYMENT
# ==============================================================================

# Copy built application from builder stage
# This copies only the compiled static assets, not source code
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom Nginx configuration
# Overrides default Nginx configuration with our optimized settings
# Includes security headers, compression, caching, and PWA support
COPY nginx/frontend.conf /etc/nginx/conf.d/default.conf

# ==============================================================================
# SECURITY AND PERFORMANCE CONFIGURATION
# ==============================================================================

# Expose HTTP port for web traffic
# Consider using 443 for HTTPS in production with SSL termination
EXPOSE 80

# ==============================================================================
# HEALTH AND MONITORING
# ==============================================================================

# Consider adding healthcheck for better container orchestration:
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#     CMD curl -f http://localhost:80/ || exit 1

# ==============================================================================
# CONTAINER EXECUTION
# ==============================================================================

# Start Nginx in foreground mode
# This keeps the container running and allows proper signal handling
# "daemon off;" is required for Docker container operation
CMD ["nginx", "-g", "daemon off;"]

# ==============================================================================
# BUILD INSTRUCTIONS
# ==============================================================================
#
# Build the frontend container:
# docker build -t rust-blog-cms-frontend .
#
# Run the container:
# docker run -p 80:80 rust-blog-cms-frontend
#
# With environment variables:
# docker run -p 80:80 \
#   -e VITE_API_URL=https://api.example.com \
#   rust-blog-cms-frontend
#
# With custom configuration:
# docker run -p 80:80 \
#   -v $(pwd)/nginx/frontend.conf:/etc/nginx/conf.d/default.conf \
#   rust-blog-cms-frontend
#

# ==============================================================================
# PRODUCTION CONSIDERATIONS
# ==============================================================================
#
# Security Recommendations:
# 1. Use HTTPS with SSL termination (load balancer or reverse proxy)
# 2. Implement rate limiting and DDoS protection
# 3. Regular security updates for base images
# 4. Scan images for vulnerabilities
# 5. Use non-root user if possible (requires Nginx config changes)
#
# Performance Optimizations:
# 1. Enable gzip/Brotli compression
# 2. Configure appropriate caching headers
# 3. Use CDN for static assets
# 4. Implement HTTP/2 or HTTP/3
# 5. Monitor resource usage and scale accordingly
#
# Monitoring and Logging:
# 1. Configure Nginx access and error logs
# 2. Implement application performance monitoring
# 3. Set up alerts for container health
# 4. Use structured logging for better analysis
#
# CI/CD Integration:
# 1. Use multi-platform builds for different architectures
# 2. Implement automated security scanning
# 3. Use semantic versioning for image tags
# 4. Implement rolling deployments for zero downtime
#
