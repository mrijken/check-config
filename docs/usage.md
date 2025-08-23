# Usage

After installation, with the next command you can check your configuration files

```shell
check-config
```

Note: with [uvx](https://docs.astral.sh/uv/guides/tools/) it is also possible to run it without installation:

```shell
uvx check-config
```

It will output nothing when the check succeeds:

```console
🥇 No violations found.
```

If you use verbose mode with -v, more will be outputted:

```console
2 checks successful.
🥇 No violations found.
```

And with -vv as option, even more output will be given:

```console
Starting check-config
Using checkers from file:///home/ubuntu/repos/check-config/example/check-config-for-usage-doc.toml
Fix: false
✅ example/check-config-for-usage-doc.toml - /home/ubuntu/.bashrc - lines_present
✅ example/check-config-for-usage-doc.toml - /home/ubuntu/.bashrc - lines_present
2 checks successful.
🥇 No violations found
```

When there fixes possible, you will get the next output.

No verbose:

```console
🪛  There is 1 violation to fix.
```

Single verhose (-v):

```console
❌ example/check-config-for-usage-doc.toml - /home/ubuntu/.bashrc - lines_present - Set file contents to:
@@ -128,3 +128,4 @@

 export EDITOR=hx
+export SHELL=/bin/bash

1 checks successful.
🪛  There is 1 violation to fix.
```

Double verbose (-vv):

```console
Starting check-config
Using checkers from example/check-config-for-usage-doc.toml
Fix: false
❌ example/check-config-for-usage-doc.toml - /home/ubuntu/.bashrc - lines_present - Set file contents to:
@@ -128,3 +128,4 @@

 export EDITOR=hx
+export SHELL=/bin/bash

✅ example/check-config-for-usage-doc.toml - /home/ubuntu/.bashrc - lines_present
1 checks successful.
🪛  There is 1 violation to fix.
```

Check Config will use the checkers as defined in `check-config.toml`. When that file is not present,
it will use `pyproject.toml` if it is present.

Optionally you can specify another path to a toml file with checkers:

```shell
check-config -p <path>
```

Optionally you can not just check your files, but also try to fix them:

```shell
check-config --fix
```

Or just view the checkers without executing them

```shell
check-config --list-checkers
```

## Pre-commit

[pre-commit](https://pre-commit.com/) helps checking your code before committing git, so you can catch errors
before the build pipeline does.

Add the next repo to the `.pre-commit-config.yaml` in your repository with the id of the hook
you want to use:

```yaml
repos:
  - repo: https://github.com/mrijken/check-config
    rev: v0.8.2
    hooks:
      # Install via Cargo and execute `check-config --fix`
      - id: check_config_fix_install_via_rust
      # Install via pip and execute `check-config --fix`
      - id: check_config_fix_install_via_python
      # Install via Cargo and execute `check-config`
      - id: check_config_check_install_via_rust
      # Install via pip and execute `check-config`
      - id: check_config_check_install_via_python
```

If you want to call check-config with other arguments, like a different toml, you can create your own hook
in your `.pre-commit-config.toml`:

```yaml
- repo: local
  hooks:
    - id: check_config_fix_install_via_rust
      name: check configuration files based on check_config.toml and try to fix them
      language: rust
      entry: check-config --fix -p check.toml -vv
      pass_filenames: false
      always_run: true
```

## Exit Codes

We use the following exit codes, which you can make use of in your build pipelines.

| code | meaning                                                                                               |
| ---- | ----------------------------------------------------------------------------------------------------- |
| 0    | OK                                                                                                    |
| 1    | Parsing error: the checkers file is not valid TOML, has a wrong check type or any other parsing error |
| 2    | Violation error: one or more of you checker have failed                                               |
