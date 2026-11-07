_default:
  just --list

[doc('generate wheel/sdist for distribution')]
build:
    uv build

[doc('Lint Python files with `ruff check`')]
check *ARGS:
    uv run ruff check {{ ARGS }}

[doc('Format Python files with `ruff format`')]
format *ARGS:
    uv run ruff format {{ ARGS }}

[doc('Run the Python test suite (rebuilds the extension if Rust changed)')]
test *ARGS:
    uv run pytest {{ ARGS }}

[doc('Run the Rust test suite')]
cargo-test *ARGS:
    cargo test --workspace --all-features {{ ARGS }}

[doc('Lint Rust code with `cargo fmt` and `cargo clippy`')]
cargo-check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings

[doc('Generate coverage report')]
test-cov *ARGS:
  just test --cov=smbmc {{ ARGS }}
  uv run coverage lcov

[doc('Build the documentation')]
docs:
  uv run sphinx-build docs docs/_build

[doc('Build and serve the documentation with live reloading on file changes.')]
docs-autobuild:
  uv run sphinx-autobuild docs docs/_build

[doc('Remove all files ignored from git')]
clean:
  git clean -fX .
