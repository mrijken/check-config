# Check Config

I have a lot of repo's. Some config for tools like ruff, is included in the repo ie via a pyproject.toml. Without
check-config it is hard to keep them in sync.

The build pipeline can check if the latest config is used via `check-config`. When I want to change the config, 
ie using a different line length, I can update the config in a repo via `check-config --fix`.

Check-config works with checker files in which you define checks, ie

```check-config.toml
# check that .venv is included in the .gitignore
[".gitignore".lines_present]
__lines__ = ".venv"
```

With `check-config` you can check (for example in a build pipeline) whether your files passed the checks.

Most checks can also be automatically fixed with `check-config --fix`, so in this case a missing line will
be added to the `.gitignore`.

The checker files can be located on a file/webserver or in a Python Package.

## Documentation

Check out the [documentation](https://check-config.readthedocs.io)
