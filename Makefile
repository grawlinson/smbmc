.DEFAULT: help
.PHONY: help clean clean-pyc clean-build dist lint test tests docs

help:
	@echo "clean         : delete all artifacts"
	@echo "clean-pyc     : delete python cache artifacts"
	@echo "clean-build   : delete distribution artifacts"
	@echo "dist          : generate distribution artifacts"
	@echo "lint          : lint with black, flake8 & reorder-python-imports"
	@echo "test          : run tests with latest Python version"
	@echo "coverage      : run coverage tests with latest Python version"
	@echo "tests         : run tests with supported Python versions"

clean: clean-pyc clean-build

clean-pyc:
	@find . -name '*.pyc' -delete
	@find . -name '*.pyo' -delete
	@find . -name __pycache__ -delete

clean-build:
	@rm --force --recursive build dist src/*.egg-info docs/_build

dist: clean
	poetry build

lint:
	nox -rs precommit

test:
	nox

coverage:
	nox -rs coverage

tests:
	nox -rs tests

docs:
	nox -rs docs