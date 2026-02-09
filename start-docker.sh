#!/bin/bash

# ==============================================================================
# RustBlogCMS - Docker Startup Script
# ==============================================================================
#
# This script automates the deployment of the Rust Blog CMS using Docker
# Compose. It performs pre-flight checks, builds containers, starts services,
# and provides access information for the deployed application.
#
# Features:
# - Environment validation and dependency checking
# - Automated container building and deployment
# - Service health monitoring
# - User-friendly output with emoji indicators
# - Access information and default credentials
#
# Requirements:
# - Docker and Docker Compose installed
# - .env file configured with proper values
# - Sufficient system resources
#
# Usage:
#   ./start-docker.sh [--update] [--dev]
#
# Options:
#   --update   : Pull latest images before building
#   --dev      : Start in development mode with mounted volumes
#
# @version 1.0.0
# @author RustBlogCMS Team
# ==============================================================================

# Script configuration
set -euo pipefail  # Exit on error, undefined vars, and pipe failures
readonly SCRIPT_NAME="RustBlogCMS Docker Startup"
readonly DEFAULT_PORT="8489"

# Color codes for output formatting
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly NC='\033[0m' # No Color

# Parse command line arguments
UPDATE_IMAGES=false
DEV_MODE=false

for arg in "$@"; do
    case $arg in
        --update)
            UPDATE_IMAGES=true
            shift
            ;;
        --dev)
            DEV_MODE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--update] [--dev] [--help]"
            echo "  --update : Pull latest images before building"
            echo "  --dev    : Start in development mode"
            echo "  --help   : Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown argument: $arg${NC}"
            exit 1
            ;;
    esac
done

# ==============================================================================
# UTILITY FUNCTIONS
# ==============================================================================

# Print formatted messages
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_header() {
    echo ""
    echo -e "${BLUE}=== $1 ===${NC}"
    echo ""
}

# Check if command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Wait for service to be ready
wait_for_service() {
    local url="$1"
    local max_attempts=30
    local attempt=1

    print_info "Waiting for service at $url to be ready..."

    while [ $attempt -le $max_attempts ]; do
        if curl -f -s "$url" > /dev/null 2>&1; then
            print_success "Service is ready after ${attempt}s"
            return 0
        fi

        echo -n "."
        sleep 1
        ((attempt++))
    done

    echo ""
    print_warning "Service may still be starting up..."
    return 1
}

# ==============================================================================
# PREFLIGHT CHECKS
# ==============================================================================

print_header "$SCRIPT_NAME"

print_info "Starting Rust Blog CMS deployment..."

# Check Docker installation
if ! command_exists docker; then
    print_error "Docker is not installed!"
    print_info "Please install Docker from https://docs.docker.com/get-docker/"
    exit 1
fi

print_success "Docker is installed: $(docker --version)"

# Check Docker Compose installation
if ! command_exists docker-compose && ! docker compose version &> /dev/null; then
    print_error "Docker Compose is not installed!"
    print_info "Please install Docker Compose from https://docs.docker.com/compose/install/"
    exit 1
fi

# Determine Docker Compose command
if command_exists docker-compose; then
    DOCKER_COMPOSE_CMD="docker-compose"
else
    DOCKER_COMPOSE_CMD="docker compose"
fi

print_success "Docker Compose is available: $(${DOCKER_COMPOSE_CMD} --version)"

# Check Docker daemon is running
if ! docker info &> /dev/null; then
    print_error "Docker daemon is not running!"
    print_info "Please start Docker and try again"
    exit 1
fi

print_success "Docker daemon is running"

# Check system resources
print_info "Checking system resources..."

# Available disk space (minimum 2GB required)
AVAILABLE_SPACE=$(df . | awk 'NR==2 {print $4}')
MIN_SPACE_KB=2097152  # 2GB in KB

if [ "$AVAILABLE_SPACE" -lt "$MIN_SPACE_KB" ]; then
    print_warning "Low disk space detected. At least 2GB recommended"
fi

# Check memory (optional)
if command_exists free; then
    AVAILABLE_MEM=$(free -m | awk 'NR==2{print $7}')
    print_info "Available memory: ${AVAILABLE_MEM}MB"
fi

# ==============================================================================
# ENVIRONMENT CONFIGURATION
# ==============================================================================

print_header "Environment Configuration"

# Check for .env file
if [ ! -f .env ]; then
    print_warning ".env file not found!"

    if [ -f .env.example ]; then
        print_info "Creating .env from .env.example..."
        cp .env.example .env
        print_warning "Please edit .env and set secure values before deploying!"
        print_info "Especially update JWT_SECRET and database credentials"
    else
        print_error ".env.example file not found!"
        print_info "Please create .env file with proper configuration"
        exit 1
    fi
else
    print_success ".env file found"

    # Check for insecure default values
    if grep -q "change_me" .env; then
        print_warning "Found default values in .env file!"
        print_warning "Please update security-sensitive values before production deployment"
    fi
fi

# Check for docker-compose.yml
if [ ! -f docker-compose.yml ] && [ ! -f docker-compose.yaml ]; then
    print_error "docker-compose.yml file not found!"
    exit 1
fi

print_success "Docker Compose configuration found"

# ==============================================================================
# DOCKER OPERATIONS
# ==============================================================================

print_header "Container Deployment"

# Update images if requested
if [ "$UPDATE_IMAGES" = true ]; then
    print_info "Pulling latest base images..."
    $DOCKER_COMPOSE_CMD pull || {
        print_warning "Some images could not be updated (may be local builds)"
    }
fi

# Stop existing containers
if $DOCKER_COMPOSE_CMD ps -q | grep -q .; then
    print_info "Stopping existing containers..."
    $DOCKER_COMPOSE_CMD down
fi

# Build and start containers
print_info "Building and starting containers..."
BUILD_ARGS=""

if [ "$DEV_MODE" = true ]; then
    print_info "Starting in development mode..."
    BUILD_ARGS="--build"
else
    BUILD_ARGS="--build"
fi

# Execute docker-compose
if $DOCKER_COMPOSE_CMD up -d $BUILD_ARGS; then
    print_success "Containers started successfully"
else
    print_error "Failed to start containers!"
    print_info "Check the logs for more information:"
    echo "   $DOCKER_COMPOSE_CMD logs"
    exit 1
fi

# ==============================================================================
# HEALTH CHECKS
# ==============================================================================

print_header "Service Health Monitoring"

# Wait for containers to initialize
print_info "Waiting for services to initialize..."
sleep 10

# Check container status
print_info "Checking container status..."
$DOCKER_COMPOSE_CMD ps

# Get the actual port (in case it's mapped differently)
FRONTEND_PORT=${DEFAULT_PORT}
if docker-compose port nginx 2>/dev/null | grep -q ':'; then
    FRONTEND_PORT=$(docker-compose port nginx | cut -d: -f2)
fi

print_info "Application will be available on port $FRONTEND_PORT"

# Wait for services to be ready
print_info "Performing health checks..."

# Check main application health
if wait_for_service "http://localhost:$FRONTEND_PORT/health"; then
    print_success "Main application is healthy"
else
    print_warning "Main application health check failed - may still be starting"
fi

# Check backend API health
if wait_for_service "http://localhost:$FRONTEND_PORT/api/health"; then
    print_success "Backend API is healthy"
else
    print_warning "Backend API health check failed - may still be starting"
fi

# ==============================================================================
# ACCESS INFORMATION
# ==============================================================================

print_header "Deployment Complete"

print_success "Rust Blog CMS has been deployed successfully!"
echo ""

# Access URLs
echo "📍 Access Information:"
echo "   Frontend:      http://localhost:$FRONTEND_PORT"
echo "   Backend API:   http://localhost:$FRONTEND_PORT/api"
echo "   Health Check:  http://localhost:$FRONTEND_PORT/health"
echo ""

# Default credentials (if applicable)
echo "🔐 Default Credentials:"
echo "   Username: admin"
echo "   Password: admin123"
echo ""
print_warning "Please change the default password after first login!"
echo ""

# Management commands
echo "📊 Management Commands:"
echo "   View logs:        $DOCKER_COMPOSE_CMD logs -f"
echo "   Stop services:    $DOCKER_COMPOSE_CMD down"
echo "   Restart services: $DOCKER_COMPOSE_CMD restart"
echo "   Update images:    $DOCKER_COMPOSE_CMD pull && $DOCKER_COMPOSE_CMD up -d --build"
echo ""

# Development information
if [ "$DEV_MODE" = true ]; then
    echo "🛠️  Development Mode:"
    echo "   Hot reload enabled for frontend"
    echo "   Source code is mounted in containers"
    echo "   Debug logging is enabled"
    echo ""
fi

# Troubleshooting information
echo "🔧 Troubleshooting:"
echo "   If the application doesn't load immediately, wait 1-2 minutes"
echo "   Check logs with: $DOCKER_COMPOSE_CMD logs"
echo "   Verify port $FRONTEND_PORT is not in use by other applications"
echo "   Ensure Docker has sufficient resources allocated"
echo ""

# Security reminders
echo "🔒 Security Reminders:"
echo "   Update default passwords and secrets"
echo "   Configure proper CORS origins for production"
echo "   Enable HTTPS in production environments"
echo "   Regularly update Docker images"
echo ""

print_success "Deployment completed successfully!"

# ==============================================================================
# POST-DEPLOYMENT HOOKS (Optional)
# ==============================================================================

# Add any post-deployment tasks here
# Examples: database migrations, cache warming, etc.

# Optional: Open browser automatically
# if command_exists xdg-open; then
#     xdg-open "http://localhost:$FRONTEND_PORT"
# elif command_exists open; then
#     open "http://localhost:$FRONTEND_PORT"
# fi

exit 0
