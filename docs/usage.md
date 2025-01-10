# Usage

After installation, with the next command you can check your configuration files

```shell
check_config
```

Note: with [uvx](https://docs.astral.sh/uv/guides/tools/) it is also possible to run it without installation:

```shell
uvx check_config
```

It will output whether the check is succeeded:

```console
Starting check-config
Fix: false
✅ dvb\devtools\check_config\black.toml - C:\Users\XXX\.pre-commit-config.yaml - entry_present
✅ dvb\devtools\check_config\black.toml - C:\Users\XXX\pyproject.toml - key_value_present
```

or not:

```console
Starting check-config
Fix: false
✅ dvb\devtools\check_config\black.toml - C:\Users\XXX\.pre-commit-config.yaml - entry_present
❌ dvb\devtools\check_config\black.toml - C:\Users\XXX\pyproject.toml - key_value_present - Set file contents to: @@ -70,7 +70,7 @@
 default = true

 [tool.black]
-line-length = 80
+line-length = 120
```

Check Config will use the checkers as defined in `check_config.toml`, but you can specify another path:

```shell
check_config -p <path>
```

Optionally you can not just check your files, but also try to fix them:

```shell
check_config --fix
```

## Pre-commit

[pre-commit](https://pre-commit.com/) helps checking your code before committing git, so you can catch errors
before the build pipeline does.

Add the next repo to the `.pre-commit-config.yaml` in your repository with the id of the hook
you want to use:

```yaml
repos:
  - repo: https://github.com/mrijken/check_config
    rev: v0.6.1
    hooks:
      # Install via Cargo and execute `check_config --fix`
      - id: check_config_fix_install_via_rust
      # Install via pip and execute `check_config --fix`
      - id: check_config_fix_install_via_python
      # Install via Cargo and execute `check_config`
      - id: check_config_check_install_via_rust
      # Install via pip and execute `check_config`
      - id: check_config_check_install_via_python
```

If you want to call check_config with other arguments, like a different toml, you can create your own hook
in your `.pre-commit-config.toml`:

```yaml
- repo: local
  hooks:
  - id: check_config_fix_install_via_rust
    name: check configuration files based on check_config.toml and try to fix them
    language: rust
    entry: check_config --fix -p check.toml -vv
    pass_filenames: false
    always_run: true
```

## Exit Codes

We use the following exit codes, which you can make use of in your build pipelines.

| code | meaning |
|------|-----------|
| 0 | OK      |
| 1 | Parsing error: the checkers file is not valid TOML, has a wrong check type or any other parsing error |
| 2 | Violation error: one or more of you checker have failed  |
