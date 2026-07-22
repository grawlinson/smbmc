_default:
  just --list

#[doc('Install all dependencies to virtual environment')]
#install-deps:
#  uv sync --all-groups

[doc('generate wheel/sdist for distribution')]
build:
    uv build

[doc('Lint Python files with `ruff check`')]
check *ARGS:
    uv run ruff check {{ ARGS }}

[doc('Format Python files with `ruff format`')]
format *ARGS:
    uv run ruff format {{ ARGS }}

[doc('Run the test suite')]
test *ARGS:
    uv run pytest {{ ARGS }}

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
