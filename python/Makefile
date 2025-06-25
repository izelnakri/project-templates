.DEFAULT_GOAL := build

VENV := .venv
PYTHON := $(VENV)/bin/python
PIP := $(VENV)/bin/pip
PYTEST := $(VENV)/bin/pytest
UVICORN_BIN := $(VENV)/bin/uvicorn
BLACK := $(VENV)/bin/black
RUFF := $(VENV)/bin/ruff

MODULE := github_user_fetcher
SERVER_MODULE := github_user_fetcher.server:app 
STAMP := $(VENV)/.installed_stamp
DEPS := pyproject.toml uv.lock

UVICORN := uvicorn $(SERVER_MODULE) --host 0.0.0.0 --port $(PORT) --reload

PORT ?= 1234
ENVIRONMENT ?= development

all: clean setup run

$(VENV):
	uv venv $(VENV)

uv.lock: pyproject.toml
	uv lock

# NOTE: $ source .venv/bin/activate # needed if I want to run $ python $script_path
setup: $(VENV) uv.lock $(STAMP)

$(STAMP): $(DEPS)
	uv pip install -e ".[dev]" --python $(PYTHON)
	@touch $(STAMP)

run: setup
	uv run cli
	# or $(PYTHON) github_user_fetcher/cli.py

run-module: setup
	$(PYTHON) -m $(MODULE)

server: setup
	$(UVICORN_BIN) $(SERVER_MODULE) --reload --port $(PORT)

server-prod: $(VENV)
	uv pip install . --python $(PYTHON)
	ENVIRONMENT=production $(UVICORN_BIN) $(SERVER_MODULE) --port $(PORT) --host 0.0.0.0 --factory

cli: setup
	$(PYTHON) cli.py

gui: setup
	$(PYTHON) gui.py

clean:
	rm -rf $(VENV) uv.lock github_user_fetcher.egg-info .mypy_cache .pytest_cache .ruff_cache
	find . -type d -name "__pycache__" -exec rm -rf {} +

format:
	@if [ -f $(BLACK) ]; then $(BLACK) .; else echo "black not found, run 'make setup'"; exit 1; fi

format-check:
	@if [ -f $(BLACK) ]; then $(BLACK) --check .; else echo "black not found, run 'make setup'"; exit 1; fi

lint:
	@if [ -f $(RUFF) ]; then $(RUFF) check .; else echo "ruff not found, run 'make setup'"; exit 1; fi

test:
	@if [ -f $(PYTEST) ]; then $(PYTEST); else echo "pytest not found, run 'make setup'"; exit 1; fi

# TODO:: in future add test-unit, test-integration and maybe test-doctest(just make it inside make test if they are fast)

# TODO: Make it create an html file/folder instead under docs:
doc: setup
	$(PYTHON) -m pydoc $(MODULE)

doc-tui: setup
	$(PYTHON) -m pydoc $(MODULE)

docker-build:
	nix build .#dockerImage

docker-run:
	sudo docker load < ./result
	sudo docker run -it --rm -p $(PORT):$(PORT) github_user_fetcher_py

docker-debug:
	sudo docker run -it --rm github_user_fetcher_py bash

flatpak-clean:
	rm -rf flatpak/build

flatpak-prepare:
	echo "Not implemented for Python yet"

flatpak-build:
	echo "Not implemented for Python yet"

flatpak-run:
	echo "Not implemented for Python yet"

