# Usage

With the next command you can check your configuration files

```shell
check_config
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

Check Config will use the checkers as defined in `checkers.toml`, but you can specify another path:

```shell
check_config -p <path>
```

Optionally you can not just check your files, but also try to fix them:

```shell
check_config --fix
```

## Exit Codes

We use the following exit codes, which you can make use of in your build pipelines.

| code | meaning |
|------|-----------|
| 0 | OK      |
| 1 | Parsing error: the checkers file is not valid TOML, has a wrong check type or any other parsing error |
| 2 | Violation error: one or more of you checker have failed  |
