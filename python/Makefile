.DEFAULT_GOAL := run

VENV := .venv
PYTHON := $(VENV)/bin/python

$(VENV):
	uv venv $(VENV)

install: $(VENV)
	@if [ ! -f requirements.lock.txt ]; then \
	  echo "🔐 Lockfile not found. Resolving dependencies..."; \
	  uv pip compile pyproject.toml --extra dev --output-file requirements.lock.txt; \
	fi
	uv pip install --requirements requirements.lock.txt --python $(PYTHON)

lock:
	uv pip compile pyproject.toml --extra dev --output-file requirements.lock.txt

update-lock:
	uv pip compile pyproject.toml --extra dev --output-file requirements.lock.txt --upgrade

clean:
	rm -rf $(VENV) requirements.lock.txt __pycache__

run: install
	$(PYTHON) hello.py
