# Contributing to StellarEscrow

Thank you for your interest in contributing to StellarEscrow! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and professional in all interactions.

## Getting Started

### Prerequisites

- Rust 1.70 or higher
- Soroban CLI
- Git
- Basic understanding of Stellar and Soroban

### Setup Development Environment

```bash
# Clone repository
git clone <your-repo-url>
cd stellar-escrow

# Install dependencies
rustup target add wasm32-unknown-unknown
cargo install soroban-cli

# Run tests
cargo test

# Build contract
cargo build --target wasm32-unknown-unknown --release
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions/improvements

### 2. Make Changes

- Write clean, readable code
- Follow Rust best practices
- Add tests for new functionality
- Update documentation as needed

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

### 4. Commit Changes

Use clear, descriptive commit messages:

```bash
git commit -m "feat: add partial release functionality"
git commit -m "fix: resolve overflow in fee calculation"
git commit -m "docs: update deployment guide"
```

Commit message format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `test:` - Tests
- `refactor:` - Code refactoring
- `chore:` - Maintenance

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Create a pull request with:
- Clear title and description
- Reference to related issues
- Test results
- Screenshots (if applicable)

## Coding Standards

### Rust Style

Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

```rust
// Good
pub fn create_trade(
    env: Env,
    seller: Address,
    buyer: Address,
    amount: u64,
) -> Result<u64, ContractError> {
    // Implementation
}

// Use descriptive variable names
let accumulated_fees = get_accumulated_fees(&env)?;

// Add comments for complex logic
// Calculate fee using basis points: fee = amount * fee_bps / 10000
let fee = amount.checked_mul(fee_bps as u64)
    .ok_or(ContractError::Overflow)?
    .checked_div(10000)
    .ok_or(ContractError::Overflow)?;
```

### Error Handling

Always use `Result` types and proper error propagation:

```rust
// Good
pub fn get_trade(env: &Env, trade_id: u64) -> Result<Trade, ContractError> {
    env.storage()
        .persistent()
        .get(&key)
        .ok_or(ContractError::TradeNotFound)
}

// Avoid panics in production code
```

### Testing

Write comprehensive tests:

```rust
#[test]
fn test_feature_name() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Setup
    let contract = create_contract(&env);
    
    // Execute
    let result = contract.function();
    
    // Assert
    assert_eq!(result, expected);
}

#[test]
#[should_panic(expected = "Error(Contract, #X)")]
fn test_error_condition() {
    // Test error cases
}
```

## What to Contribute

### High Priority

- Bug fixes
- Security improvements
- Test coverage
- Documentation improvements
- Performance optimizations

### Feature Ideas

- Multi-token support
- Batch operations
- Reputation system
- Time-locked escrow
- Partial releases
- Multi-sig arbitration

### Documentation

- Code examples
- Integration guides
- Architecture diagrams
- API documentation
- Tutorial videos

## Pull Request Process

1. **Create PR**: Submit pull request with clear description
2. **Review**: Maintainers review code and provide feedback
3. **Update**: Address review comments
4. **Approve**: PR approved by maintainer
5. **Merge**: PR merged into main branch

### PR Checklist

- [ ] Tests pass locally
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Commit messages are clear
- [ ] No merge conflicts
- [ ] Added tests for new features
- [ ] Updated CHANGELOG.md (if applicable)

## Testing Guidelines

### Test Categories

1. **Unit Tests**: Test individual functions
2. **Integration Tests**: Test complete workflows
3. **Edge Cases**: Test boundary conditions
4. **Security Tests**: Test authorization and validation
5. **Event Tests**: Verify event emission

### Test Coverage Goals

- Minimum 80% code coverage
- All public functions tested
- All error paths tested
- All state transitions tested

### Running Tests

```bash
# All tests
cargo test

# With output
cargo test -- --nocapture

# Specific test
cargo test test_create_trade

# Test coverage (requires tarpaulin)
cargo tarpaulin --out Html
```

## Documentation Guidelines

### Code Documentation

```rust
/// Creates a new trade between seller and buyer
///
/// # Arguments
/// * `seller` - Address of the seller
/// * `buyer` - Address of the buyer
/// * `amount` - Trade amount in USDC
/// * `arbitrator` - Optional arbitrator address
///
/// # Returns
/// Trade ID on success, ContractError on failure
///
/// # Errors
/// * `InvalidAmount` - If amount is zero
/// * `ArbitratorNotRegistered` - If arbitrator not registered
pub fn create_trade(
    env: Env,
    seller: Address,
    buyer: Address,
    amount: u64,
    arbitrator: Option<Address>,
) -> Result<u64, ContractError> {
    // Implementation
}
```

### README Updates

Keep README.md current with:
- Feature additions
- API changes
- Usage examples
- Deployment instructions

## Security

### Reporting Vulnerabilities

**DO NOT** open public issues for security vulnerabilities.

Instead:
1. Email security concerns to [security@example.com]
2. Include detailed description
3. Provide steps to reproduce
4. Allow time for fix before disclosure

### Security Best Practices

- Always use `require_auth()`
- Validate all inputs
- Check for overflows
- Verify state transitions
- Test authorization thoroughly

## Community

### Communication Channels

- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: General questions and ideas
- Stellar Discord: Real-time chat and support

### Getting Help

- Check existing issues and documentation
- Ask in GitHub Discussions
- Join Stellar Discord
- Review Soroban documentation

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in documentation

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Questions?

Feel free to:
- Open a GitHub Discussion
- Ask in Stellar Discord
- Comment on relevant issues

Thank you for contributing to StellarEscrow! 🚀
