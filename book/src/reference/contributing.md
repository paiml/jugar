# Contributing

We welcome contributions to Jugar!

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/paiml/jugar.git
cd jugar
```

### Install Tools

```bash
make install-tools
```

### Run Tests

```bash
make tier2
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

Follow the coding standards below.

### 3. Run Checks

```bash
# Quick check during development
make tier1

# Full check before committing
make tier2
```

### 4. Commit

```bash
git add .
git commit -m "feat: add my feature"
```

### 5. Push and Create PR

```bash
git push origin feature/my-feature
# Create PR on GitHub
```

## Coding Standards

### Zero JavaScript

**CRITICAL**: No JavaScript in game logic.

- ❌ No `.js` or `.ts` files
- ❌ No `npm` or `package.json`
- ❌ No JavaScript bundlers
- ✅ Pure Rust only
- ✅ WASM output only

### Batuta-First

Use Batuta stack components before external crates:

- `trueno` for SIMD/GPU compute
- `aprender` for ML/AI
- `trueno-viz` for rendering

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy --all-targets -- -D warnings
```

No warnings allowed.

### Tests

- Write tests for all new code
- Maintain 95%+ coverage
- Include property tests for complex logic

```bash
make coverage
```

### Documentation

- Add doc comments to public APIs
- Update book if adding features
- Include examples for new functionality

## Commit Messages

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(physics): add continuous collision detection
fix(ai): correct pathfinding edge case
docs(book): add probar chapter
```

## Pull Request Guidelines

### Requirements

All PRs must:
- Pass CI (tier2 equivalent)
- Have tests for new code
- Update documentation if needed
- Have descriptive commit messages

### Review Process

1. Automated checks run
2. Maintainer reviews code
3. Feedback addressed
4. Approval and merge

### PR Template

```markdown
## Summary

Brief description of changes.

## Changes

- Change 1
- Change 2

## Testing

How was this tested?

## Checklist

- [ ] Tests added
- [ ] Documentation updated
- [ ] `make tier2` passes
```

## Architecture Decisions

For significant changes:

1. Open an issue first
2. Discuss the approach
3. Get approval before implementing

## Code of Conduct

Be respectful and constructive. We're all here to build something great together.

## Getting Help

- Open an issue for questions
- Check existing issues first
- Join discussions on GitHub

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
