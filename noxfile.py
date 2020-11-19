"""Nox sessions."""
import functools
import os
from pathlib import Path
from textwrap import dedent

import nox
import nox_poetry
from nox.sessions import Session

# package name
package = "smbmc"

# python versions
supported_versions = ["3.6", "3.7", "3.8", "3.9"]
latest_version = supported_versions[-1]

# nox settings
nox.options.sessions = [f"tests-{latest_version}"]
nox.options.reuse_existing_virtualenvs = True

locations = ["src", "tests", "noxfile.py", "docs/conf.py"]


@nox.session(python=supported_versions)
def tests(session: Session) -> None:
    """Run the test suite.

    Args:
        session: The Session object.
    """
    nox_poetry.install(session, nox_poetry.WHEEL)
    nox_poetry.install(session, "pytest", "betamax")
    session.run("pytest")


@nox.session(python=latest_version)
def coverage(session: Session) -> None:
    """Generate coverage report.

    Args:
        session: The Session object.
    """
    nox_poetry.install(session, nox_poetry.WHEEL)
    nox_poetry.install(session, "pytest", "betamax", "pytest-cov", "coverage[toml]")
    session.run("pytest", f"--cov={package}", "tests/")


def activate_virtualenv_in_precommit_hooks(session: Session) -> None:
    """Activate virtualenv in hooks installed by pre-commit.

    This function patches git hooks installed by pre-commit to activate the
    session's virtual environment. This allows pre-commit to locate hooks in
    that environment when invoked from git.

    Args:
        session: The Session object.
    """
    if session.bin is None:
        return

    virtualenv = session.env.get("VIRTUAL_ENV")
    if virtualenv is None:
        return

    hookdir = Path(".git") / "hooks"
    if not hookdir.is_dir():
        return

    for hook in hookdir.iterdir():
        if hook.name.endswith(".sample") or not hook.is_file():
            continue

        text = hook.read_text()
        bindir = repr(session.bin)[1:-1]  # strip quotes
        if not (
            Path("A") == Path("a") and bindir.lower() in text.lower() or bindir in text
        ):
            continue

        lines = text.splitlines()
        if not (lines[0].startswith("#!") and "python" in lines[0].lower()):
            continue

        header = dedent(
            f"""\
            import os
            os.environ["VIRTUAL_ENV"] = {virtualenv!r}
            os.environ["PATH"] = os.pathsep.join((
                {session.bin!r},
                os.environ.get("PATH", ""),
            ))
            """
        )

        lines.insert(1, header)
        hook.write_text("\n".join(lines))


@nox.session(python=latest_version)
def precommit(session: Session) -> None:
    """Lint using pre-commit.

    Args:
        session: The Session object.
    """
    args = session.posargs or [
        "run",
        "--all-files",
    ]  # "--show-diff-on-failure"]
    nox_poetry.install(
        session,
        "black",
        "darglint",
        "flake8",
        "flake8-bandit",
        "flake8-bugbear",
        "flake8-docstrings",
        "flake8-rst-docstrings",
        "pep8-naming",
        "pre-commit",
        "pre-commit-hooks",
        "reorder-python-imports",
    )
    session.run("pre-commit", *args)
    if args and args[0] == "install":
        activate_virtualenv_in_precommit_hooks(session)


@nox.session(python=latest_version)
def docs(session: Session) -> None:
    """Build the documentation.

    Args:
        session: The Session object.
    """
    output_dir = os.path.join(session.create_tmp(), "output")
    doctrees, html = map(
        functools.partial(os.path.join, output_dir), ["doctrees", "html"]
    )
    session.run("rm", "-rf", output_dir, external=True)

    nox_poetry.install(session, nox_poetry.WHEEL)
    nox_poetry.install(session, "sphinx", "sphinx-autobuild", "sphinx-rtd-theme")
    session.cd("docs")
    sphinx_args = [
        "-b",
        "html",
        "-W",
        "-d",
        doctrees,
        ".",
        html,
    ]

    if not session.interactive:
        sphinx_cmd = "sphinx-build"
    else:
        sphinx_cmd = "sphinx-autobuild"
        sphinx_args.insert(0, "--open-browser")

    session.run(sphinx_cmd, *sphinx_args)
