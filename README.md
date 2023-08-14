# Check Config

It can can be cumbersome when you have multiple projects and environments with configuration files which need to be
upgraded and keep in sync regulary. Check-config will help you.

## Installation

The preferred installation is via pip(x):

```shell
pip(x) install check_config
```

## Usage

With the next command you can check your configuration files

```shell
check_config
```

This will use the checkers as defined in `checkers.toml`, but you can specify another path:

```shell
check_config -p <path>
```

Optionally you can not just check your files, but also try to fix them:

```shell
check_config --fix
```

## Checkers

CheckConfig uses `checkers` which define the desired state of the configuration files. There are several
checker types (and more to come):

| name | description | fixable |
|------|-------------|---------|
| [file_absent](#file-absent) |  the file must be absent | yes |
| [file_present](#file-present) |  the file must be present, indifferent the content | yes |
| [key_absent](#key-absent) | a specified key must be absent in a toml / yaml / json file  | yes |
| [key_value_present](#key-value-present) | a specified key with a specified value must be present in a toml / yaml / json file  | yes |
| [key_value_regex_match](#key-value-regex-match) | the value of a specified key must be match the specified regex in a toml / yaml / json file  | no |
| [lines_absent](#lines-absent) | the specified lines must be absent | yes |
| [lines_present](#lines-present) | the specified lines must be present | yes |

### Checker.toml

The `checkers.toml` consist of zero or one `check-config` tables:

```toml
[check-config]
additional_checks = []  # optional list of toml files with additional checks
```

And one or more checkers

```toml
["<file_path>".<checker_name>.<checker_keys>]
key = value
```

The syntax is slightly different per check type/

### File Absent

`file_absent` will check if the file is absent.

The next example will check that `test/absent_file` will be absent.

```toml
["test/absent_file".file_absent]
```

### File Present

`file_present` will check if the file is present.

The next example will check that `test/present_file` will be present. It will
not check the contents.

```toml
["test/present_file".file_absent]
```

## Key Absent

```toml
["test/present1.toml".key]
```