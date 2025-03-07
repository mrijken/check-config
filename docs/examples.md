# Examples

## Python

```toml
[check-config]
include = ["black.toml", "mypy.toml", "tuff.toml"]

# Do not use setup.cfg
["setup.cfg".file_absent]

# Do not use setup.py
["setup.py".file_absent]

# Do not use requirements.txt
["requirements.txt".file_absent]

# use poetry as build tool
["pyproject.toml".key_value_present.build-system]
requires = ["poetry-core>=1.0.0"]
build-backend = "poetry.core.masonry.api"

# prevent from adding .venv and cache to git
[[".gitignore".lines_present]]
__lines__ = "__pycache__"

[[".gitignore".lines_present]]
__lines__ = ".cache"

[[".gitignore".lines_present]]
__lines__ = ".venv"
```

## Ruff

```toml
["pyproject.toml".key_value_present.tool.ruff]
line-length = 120
select = ["ALL"]
ignore = [
    "D",      # pydocstyle
    "ANN102", # Missing type annotation for {name} in classmethod
    "ANN101", # Missing type annotation for `self` in method
    "EM102",  # Exception must not use an f-string literal, assign to variable first
    "TCH",    # Use Type Checking Block
    "TRY003", # Avoid specifying long messages outside the exception class
    "EM",     # Error messages must not use string literal, must not use f-string
    "FBT",    # Do not use positional / default boolean arguments
]

["pyproject.toml".key_value_regex_match.tool.ruff]
target-version = "(py310)|(py311)"

["pyproject.toml".key_value_present.tool.ruff.per-file-ignores]
"tests/**" = [
    "S101",    # No usage of assert
    "INP",     # No implicit namespace packages
    "SLF",     # No private member accessed
    "ARG",     # unused arguments
    "PLR2004", # No usage of magic contants
    "ANN201",  # Missing return type annotation
]
"notebooks/**" = ["ALL"]

["pyproject.toml".key_value_present.tool.ruff.isort]
force-single-line = true
```

## Black

```toml
["pyproject.toml".key_value_present.tool.black]
line-length = 120
exclude = "(/(notebooks|\\.git|\\.venv)/)"

[".pre-commit-config.yaml".entry_present.repos]
__items__ = [
    { repo = "local", hooks = [
        { id = "black", name = "black", language = "system", entry = "poetry run black .", pass_filenames = false, always_run = true },
    ] },
]
```

## Mypy

```toml
["pyproject.toml".key_value_present.tool.mypy]
explicit_package_bases = true
namespace_packages = true
ignore_missing_imports = true

[".pre-commit-config.yaml".entry_present.repos]
__items__ = [
    { repo = "local", hooks = [
        { id = "mypy", name = "mypy", language = "system", entry = "poetry run mypy dvb", pass_filenames = false, always_run = true },
    ] },
    { repo = "local", hooks = [
        { id = "mypy_on_tests", name = "mypy", language = "system", entry = "poetry run mypy tests", pass_filenames = false, always_run = true },
    ] },
]

[[".gitignore".lines_present]]
__lines__ = ".mypy_cache"
```

## Bashrc

```toml
[".bashrc".file_regex_match]
__regex__ = "export KEY=.*"
```

```toml
[".bashrc".lines_present]
__lines__ = "export KEY=1"
```


## .env files

```toml
[".env".file_regex_match]
__regex__ = "KEY=.*"
```


```toml
[".env".lines_present]
__lines__ = "KEY=1"
```
