# Check Config

It can be cumbersome when you have multiple projects and environments with configuration files which need to be
upgraded and keep in sync regularly. Check-config will help you with i.e. making sure that the configuration
file have the (upgraded) settings.

Check-config works with checker files in which you define checks, ie

```check_config.toml
# check that .venv is included in the .gitignore
[".gitignore".lines_present]
__lines__ = ".venv"
```

With `check-config` you can check (for example in a build pipeline) whether your files passed the checks.

Most checks can also be automatically fixed with `check-config --fix`, so in this case a missing line will
be added to the `.gitignore`.

A large number of [file types](features/#file-types) and [checks](checkers) are supported or will
be supported in the near [future](features).
