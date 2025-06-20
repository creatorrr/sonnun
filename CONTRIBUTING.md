# Contributing to Sonnun

Thank you for your interest in contributing to Sonnun! This document provides guidelines and
instructions for contributing to the project.

## 🚀 Getting Started

1. **Fork the repository** and clone it locally
2. **Install dependencies**:
   ```bash
   npm install
   cd src-tauri && cargo build
   ```
3. **Set up your environment**:
   ```bash
   export OPENAI_API_KEY="your-api-key-here"
   ```

## 📋 Development Workflow

### Code Quality Tools

We use several tools to maintain code quality:

- **ESLint** - JavaScript/TypeScript linting
- **Prettier** - Code formatting
- **TypeScript** - Type checking
- **Husky** - Git hooks
- **lint-staged** - Run linters on staged files

### Available Scripts

```bash
# Development
npm run dev              # Frontend development server
npm run tauri:dev        # Full app development

# Code Quality
npm run lint             # Run ESLint
npm run lint:fix         # Fix ESLint issues
npm run format           # Format code with Prettier
npm run format:check     # Check formatting
npm run type-check       # TypeScript type checking

# Testing
npm run test             # Run tests
npm run test:watch       # Run tests in watch mode
npm run test:coverage    # Generate coverage report
npm run test:rust        # Run Rust tests

# Building
npm run build            # Build frontend
npm run tauri:build      # Build desktop app
```

### Pre-commit Hooks

Our pre-commit hooks automatically:

1. Run ESLint on staged TypeScript/JavaScript files
2. Format code with Prettier
3. Ensure all tests pass

If the hooks fail, fix the issues and try committing again.

## 🏗️ Project Structure

```
sonnun/
├── src/                    # React frontend
│   ├── components/         # React components
│   ├── extensions/         # Tiptap extensions
│   ├── hooks/             # Custom React hooks
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend
│   ├── src/               # Rust source code
│   └── target/            # Build output
├── badge-service/         # Verification badge service
└── public-key-service/    # Public key distribution
```

## 📝 Coding Standards

### TypeScript/React

- Use functional components with TypeScript
- Follow the existing code style (enforced by ESLint/Prettier)
- Write meaningful component and variable names
- Add JSDoc comments for complex functions
- Use proper TypeScript types (avoid `any`)

### Rust

- Follow Rust naming conventions (snake_case)
- Use `Result<T, String>` for Tauri commands
- Add documentation comments for public functions
- Handle errors gracefully

### Git Commits

We follow conventional commit messages:

```
feat: add new feature
fix: fix bug
docs: update documentation
style: formatting changes
refactor: code refactoring
test: add tests
chore: maintenance tasks
```

## 🧪 Testing

- Write tests for new features
- Ensure all tests pass before submitting PR
- Aim for good test coverage
- Test both happy paths and edge cases

### Writing Tests

Frontend tests go in `__tests__` directories or `.test.ts` files:

```typescript
// src/utils/__tests__/example.test.ts
describe('ExampleFunction', () => {
  it('should handle normal input', () => {
    expect(exampleFunction('input')).toBe('expected')
  })
})
```

## 🔄 Pull Request Process

1. **Create a feature branch**: `git checkout -b feature/your-feature`
2. **Make your changes** following the coding standards
3. **Write/update tests** for your changes
4. **Run all checks locally**:
   ```bash
   npm run lint
   npm run format:check
   npm run type-check
   npm run test
   ```
5. **Commit your changes** with a descriptive message
6. **Push to your fork** and create a pull request
7. **Fill out the PR template** completely
8. **Wait for review** and address any feedback

### PR Requirements

- All tests must pass
- No linting errors
- Code is properly formatted
- TypeScript compilation succeeds
- Clear description of changes
- Screenshots for UI changes

## 🐛 Reporting Issues

When reporting issues, please include:

1. **Description** of the problem
2. **Steps to reproduce**
3. **Expected behavior**
4. **Actual behavior**
5. **Environment details** (OS, Node version, etc.)
6. **Screenshots** if applicable

## 💡 Feature Requests

We welcome feature requests! Please:

1. Check existing issues first
2. Provide a clear use case
3. Explain why this feature would be valuable
4. Be open to discussion and feedback

## 📚 Additional Resources

- [Project README](README.md)
- [Implementation Guide](IMPLEMENTATION.md)
- [Developer Guide](CLAUDE.md)
- [Tauri Documentation](https://tauri.app/)
- [Tiptap Documentation](https://tiptap.dev/)

## 🤝 Code of Conduct

Please be respectful and constructive in all interactions. We're building something together!

## 📄 License

By contributing, you agree that your contributions will be licensed under the project's MIT License.

---

Thank you for contributing to Sonnun! Your efforts help make honest content attribution easier for
everyone.
