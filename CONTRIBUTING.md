# Contributing to HeaderCraft

Thank you for contributing to HeaderCraft!

## Development Setup

We use `mise` to manage toolchains and `hk` for pre-commit linting.

```bash
# Install toolchains
mise install

# Run the test suite
cargo test

# Run linting and formatting
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check

# Run hooks via hk
hk check --all
```

## Pull Request Guidelines

1. **Keep it focused:** Each pull request should address a single feature or bug fix.
2. **Add tests:** Include unit tests in `src/` or CLI integration tests in `tests/cli_tests.rs`.
3. **Preserve formatting:** Run `cargo fmt` before submitting.
4. **Zero clippy warnings:** Ensure `cargo clippy --all-targets -- -D warnings` passes.
5. **Update documentation:** If adding or changing configuration options, update `README.md` and `AGENTS.md`.

## Commit Messages

We adhere to the [Conventional Commits](https://www.conventionalcommits.org/) specification:

- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation updates
- `test:` Test additions or refactoring
- `chore:` Maintenance, dependency bumps, or CI updates
