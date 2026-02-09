# package.json Documentation

## Overview
This is the main Node.js package configuration file for the RustBlogCMS project. It defines project metadata, dependencies, scripts, and build configurations for the React frontend.

## File Purpose
- Defines the project as a Node.js package with ES modules support
- Manages all frontend dependencies and development tools
- Provides npm scripts for development, testing, building, and documentation
- Configures the project for Vite-based development and build workflow

## Project Metadata

### Basic Information
- **name**: "rust-blog-cms" - The package name used in npm registry
- **version**: "1.0.0" - Current semantic version of the project
- **description**: A modern, fully customizable CMS for creating beautiful tutorial websites with React and Rust
- **author**: "zerox80" - Project author/maintainer
- **license**: "MIT" - Software license type
- **type**: "module" - Enables ES modules support for modern JavaScript

### Repository Configuration
```json
"repository": {
  "type": "git",
  "url": "https://github.com/zerox80/RustBlogCMS"
}
```
- Defines the Git repository location for cloning and contributions
- Used by npm and other tools for source code management

### Keywords
Project tags for npm registry discoverability:
- cms, tutorial, linux - Core functionality
- react, rust - Technology stack
- education, learning-platform - Domain
- content-management, admin-panel - Features

## Scripts Configuration

### Development Scripts
- **dev**: `vite` - Starts the Vite development server with hot reload
- **build**: `vite build` - Creates production-optimized build in dist/ directory
- **preview**: `vite preview` - Serves the production build locally for testing

### Code Quality Scripts
- **lint**: `eslint src --ext js,jsx` - Runs ESLint on JavaScript and JSX files
- **test**: `vitest` - Runs unit tests using Vitest
- **test:ui**: `vitest --ui` - Runs tests with visual interface
- **test:coverage**: `vitest run --coverage` - Generates test coverage reports
- **test:e2e**: `playwright test` - Runs end-to-end tests
- **test:e2e:ui**: `playwright test --ui` - Runs E2E tests with visual interface

### Content Management Scripts
- **export-content**: Rust backend utility for exporting tutorial content
- **import-content**: Rust backend utility for importing tutorial content

### Documentation Scripts
- **docs:check**: Validates JSDoc comments and linting
- **docs:coverage**: Checks documentation coverage (95% threshold)
- **docs:lint**: Lints documentation formatting
- **docs:generate**: Builds HTML documentation from code comments
- **docs:serve**: Serves documentation on port 4000
- **docs:build**: Combines generation and coverage checking
- **docs:verify**: Full documentation validation
- **docs:watch**: Serves documentation with live reload

## Dependencies

### Production Dependencies
Core runtime dependencies required for the application:

#### Security & Content Processing
- **dompurify**: ^3.2.2 - HTML sanitization to prevent XSS attacks
- **highlight.js**: ^11.10.0 - Syntax highlighting for code blocks
- **katex**: ^0.16.25 - Mathematical typesetting engine

#### React Ecosystem
- **react**: ^18.3.1 - Core React library
- **react-dom**: ^18.3.1 - React DOM renderer
- **react-router-dom**: ^7.9.5 - Client-side routing
- **react-helmet-async**: ^2.0.5 - Async head tag management for SEO

#### Internationalization
- **i18next**: ^23.11.5 - Internationalization framework
- **react-i18next**: ^15.1.5 - React bindings for i18next

#### Markdown & Content Rendering
- **react-markdown**: ^10.1.0 - Markdown to React component renderer
- **rehype-highlight**: ^7.0.2 - Syntax highlighting plugin
- **rehype-katex**: ^7.0.1 - LaTeX math plugin
- **rehype-raw**: ^7.0.0 - HTML in markdown support
- **remark-breaks**: ^4.0.0 - Line break handling
- **remark-gfm**: ^4.0.1 - GitHub Flavored Markdown
- **remark-math**: ^6.0.0 - Math notation support
- **remark-parse**: ^11.0.0 - Markdown parser
- **unist-util-visit**: ^5.0.0 - AST traversal utilities

#### UI Components
- **lucide-react**: ^0.552.0 - Icon library
- **prop-types**: ^15.8.1 - React prop type validation

### Development Dependencies
Build tools, testing frameworks, and development utilities:

#### Testing & Quality Assurance
- **@playwright/test**: ^1.49.1 - End-to-end testing framework
- **@testing-library/jest-dom**: ^6.6.3 - Jest DOM matchers
- **@testing-library/react**: ^16.1.0 - React testing utilities
- **@testing-library/user-event**: ^14.5.2 - User interaction simulation
- **vitest**: ^2.1.8 - Unit testing framework (via Vite)
- **@vitest/coverage-v8**: ^2.1.8 - Code coverage reporting
- **@vitest/ui**: ^2.1.8 - Visual test interface
- **jsdom**: ^25.0.1 - DOM implementation for testing

#### Build & Development Tools
- **vite**: ^7.1.12 - Build tool and dev server
- **@vitejs/plugin-react**: ^5.1.0 - React plugin for Vite
- **tailwindcss**: ^4.1.16 - CSS framework
- **@tailwindcss/postcss**: ^4.0.0 - Tailwind PostCSS integration
- **postcss**: ^8.5.6 - CSS transformation tool
- **autoprefixer**: ^10.4.21 - CSS vendor prefixing

#### Code Quality & Linting
- **eslint**: ^9.18.0 - JavaScript linter
- **eslint-plugin-react**: ^7.37.3 - React-specific linting rules
- **eslint-plugin-react-hooks**: ^5.1.0 - React Hooks linting

#### Development Utilities
- **patch-package**: ^8.0.0 - Node module patching
- **@types/react**: ^18.3.5 - React TypeScript definitions
- **@types/react-dom**: ^18.3.2 - React DOM TypeScript definitions

## Security Considerations

### Dependency Security
- **dompurify**: Critical for XSS prevention in user-generated content
- Regular security updates required for all dependencies
- Use `npm audit` to check for known vulnerabilities
- Consider automated security scanning in CI/CD pipeline

### Content Security
- Sanitize all user input before rendering
- Configure Content Security Policy headers
- Validate file uploads and content types

## Performance Implications

### Bundle Size Optimization
- Tree shaking enabled through ES modules
- Code splitting configured in Vite
- Lazy loading for heavy dependencies
- Production builds optimized with minification

### Development Performance
- Vite provides fast HMR (Hot Module Replacement)
- ES modules enable fast development builds
- Source maps available for debugging

## Usage Instructions

### Development Setup
```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Run tests
npm test

# Build for production
npm run build
```

### Content Management
```bash
# Export tutorial content
npm run export-content

# Import tutorial content
npm run import-content
```

### Documentation Workflow
```bash
# Generate and verify documentation
npm run docs:build

# Serve documentation locally
npm run docs:serve

# Check documentation coverage
npm run docs:coverage
```

## Dependencies and Requirements

### System Requirements
- Node.js 18+ (ES modules support required)
- npm or yarn package manager
- Rust toolchain (for backend content utilities)

### External Dependencies
- Cargo/Rust for backend utilities
- Docker for containerized deployment
- Nginx for reverse proxy (production)

## Best Practices

### Dependency Management
- Use semantic versioning for dependency updates
- Pin exact versions for critical dependencies
- Regular security audits and updates
- Monitor bundle size with each dependency addition

### Script Optimization
- Use specific test suites for different testing needs
- Implement pre-commit hooks for quality checks
- Configure CI/CD to run appropriate scripts

### Documentation Maintenance
- Keep documentation coverage above 95%
- Update docs when adding new features
- Use JSDoc for API documentation
- Maintain consistent documentation style