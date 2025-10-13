#!/bin/bash
uv venv
uv pip install --upgrade --no-cache-dir mkdocs
uv pip install -r docs/requirements.txt
uv run mkdocs build --clean --site-dir .docs/html --config-file mkdocs.yml
rm uv.lock
rm -r .venv
