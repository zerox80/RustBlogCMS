# Contributing to RustBlogCMS

Thank you for your interest in contributing to RustBlogCMS! This guide will help you understand our development practices, coding standards, and documentation requirements.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Documentation Requirements](#documentation-requirements)
4. [Code Standards](#code-standards)
5. [Testing Requirements](#testing-requirements)
6. [Security Guidelines](#security-guidelines)
7. [Pull Request Process](#pull-request-process)

## Getting Started

### Prerequisites

- Node.js 18+
- Rust toolchain 1.82+
- Git
- Familiarity with React, Rust, and modern web development

### Initial Setup

1. **Fork the repository**
   ```bash
   git clone https://github.com/your-username/RustBlogCMS.git
   cd RustBlogCMS
   ```

2. **Install dependencies**
   ```bash
   # Frontend dependencies
   npm install
   
   # Backend dependencies (if working on Rust code)
   cd backend
   cargo fetch
   cd ..
   ```

3. **Set up development environment**
   ```bash
   # Copy environment files
   cp backend/.env.example backend/.env
   cp .env.example .env.local
   
   # Start development servers
   npm run dev
   ```

## Development Workflow

### Branch Strategy

- **Main branch**: `main` - Always stable and deployable
- **Develop branch**: `develop` - Integration branch for features
- **Feature branches**: `feature/feature-name` - Isolated development
- **Bugfix branches**: `bugfix/description` - Bug fixes
- **Hotfix branches**: `hotfix/critical-fix` - Emergency fixes

### Git Workflow

1. **Create feature branch**
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/your-feature-name
   ```

2. **Make changes with regular commits**
   ```bash
   # Commit with conventional messages
   git commit -m "feat: add user authentication"
   git commit -m "docs: update API documentation"
   git commit -m "fix: resolve login timeout issue"
   ```

3. **Keep branch updated**
   ```bash
   git fetch origin
   git rebase origin/develop
   ```

4. **Create pull request**
   ```bash
   git push origin feature/your-feature-name
   # Create PR through GitHub interface
   ```

## Documentation Requirements

### Documentation Standards

All contributions must include comprehensive documentation following our [Documentation Standards](DOCUMENTATION_STANDARDS.md).

#### Before Submitting

- [ ] All new functions/components have comprehensive JSDoc/rustdoc
- [ ] Documentation includes security considerations
- [ ] Examples are provided and tested
- [ ] Performance implications are documented
- [ ] Cross-references to related code are included
- [ ] Version compatibility is specified

#### Documentation Coverage

- **Rust**: 100% of public items must be documented
- **JavaScript**: 95% of exported functions/components must be documented
- **Examples**: Every documented function must include working examples
- **Security**: All user input handling must document security measures

### Documentation Templates

Use our established templates for consistency:

#### Rust Function Template
```rust
/// Brief description of function's purpose.
/// 
/// Detailed explanation of behavior, algorithms, and edge cases.
/// 
/// # Arguments
/// * `param1` - Description with validation rules
/// * `param2` - Description with security considerations
/// 
/// # Returns
/// * `Ok(Type)` - Description of successful return
/// * `Err(Error)` - Description of error conditions
/// 
/// # Security Considerations
/// - Input validation approaches
/// - Attack vector prevention
/// - Resource protection measures
/// 
/// # Examples
/// ```rust,no_run
/// let result = function_name(param1, param2)?;
/// assert!(result.is_ok());
/// ```
```

#### JavaScript Component Template
```jsx
/**
 * Brief description of component's purpose.
 * 
## Features
 * - **Feature 1**: Description with implementation details
 * - **Feature 2**: Description with security considerations
 * 
 * ## Security Considerations
 * - XSS prevention measures
 * - Input sanitization approaches
 * - Safe rendering practices
 * 
 * @param {Object} props - Component properties
 * @param {string} props.requiredProp - Required property with validation
 * @param {string} [props.optionalProp] - Optional property with defaults
 * 
 * @returns {JSX.Element} Rendered component
 * 
 * @example
 * ```jsx
 * <ComponentName requiredProp="value" optionalProp="optional" />
 * ```
 */
```

## Code Standards

### Rust Code Standards

#### Formatting
- Use `rustfmt` for consistent formatting
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Use clippy for linting: `cargo clippy -- -D warnings`

#### Documentation
- Every public function must have `///` doc comments
- Module-level documentation with `//!`
- Include examples that can be tested with `cargo test --doc`

#### Error Handling
- Use `Result<T, Error>` for fallible operations
- Provide meaningful error messages
- Include error context where helpful

#### Security
- Validate all inputs at API boundaries
- Use parameterized queries for database operations
- Implement rate limiting for public endpoints

### JavaScript Code Standards

#### Formatting
- Use Prettier for consistent formatting: `npm run lint:fix`
- Follow ESLint rules: `npm run lint`
- Use modern ES6+ features appropriately

#### React Patterns
- Use functional components with hooks
- Implement proper prop validation with PropTypes
- Follow accessibility guidelines (ARIA attributes, semantic HTML)

#### Security
- Sanitize user inputs with DOMPurify
- Use Content Security Policy headers
- Prevent XSS in dynamic content rendering

## Testing Requirements

### Rust Testing

#### Unit Tests
- Test all public functions
- Include edge cases and error conditions
- Use property-based testing where appropriate
- Maintain >90% code coverage

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_success_case() {
        // Test happy path
    }

    #[test]
    fn test_function_error_cases() {
        // Test error conditions
    }

    #[test]
    fn test_function_edge_cases() {
        // Test boundary conditions
    }
}
```

#### Integration Tests
- Test API endpoints with realistic data
- Include authentication and authorization scenarios
- Test error responses and status codes
- Use test databases for isolation

### JavaScript Testing

#### Unit Tests
- Test all exported functions and components
- Mock external dependencies
- Test user interactions and events
- Maintain >85% code coverage

```jsx
import { render, screen, fireEvent } from '@testing-library/react';
import Component from './Component';

describe('Component', () => {
  test('renders with required props', () => {
    render(<Component requiredProp="test" />);
    expect(screen.getByText('test')).toBeInTheDocument();
  });

  test('handles user interactions', () => {
    const handleClick = jest.fn();
    render(<Component requiredProp="test" onClick={handleClick} />);
    
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });
});
```

#### End-to-End Tests
- Test critical user journeys
- Include accessibility testing
- Test responsive design
- Use Playwright for cross-browser testing

### Running Tests

```bash
# Frontend tests
npm test                    # Run unit tests
npm run test:coverage       # Run with coverage
npm run test:e2e          # Run end-to-end tests

# Backend tests
cd backend
cargo test                 # Run all tests
cargo test --doc          # Test documentation examples
cargo test --release        # Run optimized tests
```

## Security Guidelines

### Input Validation

#### Rust Backend
- Validate all user inputs at API boundaries
- Use type-safe validation libraries
- Sanitize database inputs
- Implement rate limiting

```rust
// Example input validation
fn validate_user_input(input: &str) -> Result<String, ValidationError> {
    if input.len() > MAX_LENGTH {
        return Err(ValidationError::TooLong);
    }
    
    if !is_valid_characters(input) {
        return Err(ValidationError::InvalidCharacters);
    }
    
    Ok(input.to_string())
}
```

#### JavaScript Frontend
- Sanitize HTML content with DOMPurify
- Validate form inputs before submission
- Use Content Security Policy
- Prevent XSS in dynamic rendering

```jsx
// Example safe rendering
import DOMPurify from 'dompurify';

const SafeHtmlRenderer = ({ content }) => {
  const cleanContent = DOMPurify.sanitize(content);
  return <div dangerouslySetInnerHTML={{ __html: cleanContent }} />;
};
```

### Authentication & Authorization

- Use JWT tokens with proper expiration
- Implement CSRF protection
- Use secure cookie flags
- Validate user permissions on every request

### Data Protection

- Encrypt sensitive data at rest
- Use HTTPS for all communications
- Implement proper logging and monitoring
- Follow GDPR and privacy regulations

## Pull Request Process

### Pre-Commit Checklist

Before creating a pull request, ensure:

- [ ] Code follows project standards
- [ ] All tests pass locally
- [ ] Documentation is comprehensive and updated
- [ ] Security considerations are addressed
- [ ] Performance impact is considered
- [ ] Accessibility requirements are met
- [ ] Commit messages follow conventional format

### Pull Request Template

```markdown
## Description
Brief description of changes and their purpose.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Security enhancement

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] E2E tests added/updated
- [ ] Manual testing completed

## Documentation
- [ ] Code documentation updated
- [ ] README updated (if applicable)
- [ ] API documentation updated
- [ ] User documentation updated

## Security
- [ ] Input validation implemented
- [ ] Security considerations documented
- [ ] Security review completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Testing coverage maintained
- [ ] No breaking changes (or documented)
- [ ] Ready for review
```

### Review Process

1. **Automated Checks**
   - CI/CD pipeline runs tests and linting
   - Documentation coverage verified
   - Security scans performed
   - Performance benchmarks run

2. **Peer Review**
   - At least one maintainer approval required
   - Focus on security, performance, and architecture
   - Documentation quality assessed
   - Test coverage verified

3. **Merge Requirements**
   - All automated checks pass
   - Documentation is complete
   - Security review approved
   - Tests maintain coverage requirements

### Merge Types

- **Squash and merge** for feature branches
- **Rebase and merge** for hotfixes
- **Merge commits** for develop branch synchronization

## Getting Help

### Resources

- [Documentation Standards](DOCUMENTATION_STANDARDS.md)
- [API Documentation](./docs/api/)
- [Architecture Overview](./docs/architecture/)
- [Security Guidelines](./docs/security/)

### Communication

- **Issues**: Use GitHub issues for bug reports and feature requests
- **Discussions**: Use GitHub Discussions for questions and ideas
- **Code Review**: Participate in PR reviews to improve code quality

### Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please:

- Be respectful and constructive
- Welcome newcomers and help them learn
- Focus on what is best for the community
- Show empathy towards other community members

## Recognition

Contributors are recognized for their valuable input through:

- **Contributors section** in README
- **Release notes** highlighting significant contributions
- **GitHub badges** for various contribution types
- **Community appreciation** in project communications

Thank you for contributing to RustBlogCMS! Your contributions help make this project better for everyone.
