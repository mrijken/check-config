# Checkers

Check Config uses `checkers` which define the desired state of the configuration files. There are several
checker types (and more to come):

| name | description | fixable |
|------|-------------|---------|
| [file_absent](#file-absent) |  the file must be absent | yes |
| [file_present](#file-present) |  the file must be present, indifferent the content | yes |
| [key_absent](#key-absent) | a specified key must be absent in a toml / yaml / json file  | yes |
| [key_value_present](#key-value-present) | a specified key with a specified value must be present in a toml / yaml / json file  | yes |
| [key_value_regex_match](#key-value-regex-match) | the value of a specified key must be match the specified regex in a toml / yaml / json file  | no |
| [entry_absent](#entry-absent) | a specified entry must be absent in the array of a toml / yaml / json file  | yes |
| [entry_present](#entry-present) | a specified entry  must be present in the of a toml / yaml / json file  | yes |
| [lines_absent](#lines-absent) | the specified lines must be absent | yes |
| [lines_present](#lines-present) | the specified lines must be present | yes |

## Checker.toml

The `checkers.toml` consist of zero or one `check-config` tables with configuration for check-config itself:

```toml
[check-config]
additional_checks = [  # optional list of toml files with additional checks
    "/home/me/.checkers/check.toml",  # absolute path
    "~/.checkers/check.toml",  # relative to home dir of current user
    "check.toml", # relative (to the parent toml) path
    "py://my_package:checkers/python.toml", # path to file in python package
]
```

And one or more checkers

```toml
["<file_path>".<checker_name>.<checker_keys>]
key = "value"
```

The syntax is slightly different per check type. See the next sections for help about the checker definitions.

You can use arrays of toml tables when when a check has to be done more than once, ie:

```toml
[[".gitignore".lines_present]]
__lines__ = "__pycache__"

[[".gitignore".lines_present]]
__lines__ = ".cache"
```

When using a path to a Python package to include checkers, the activated Python (virtual) environment will be used.

## File Absent

`file_absent` will check if the file is absent.

The next example will check that `test/absent_file` will be absent.

```toml
["test/absent_file".file_absent]
```

## File Present

`file_present` will check if the file is present.

The next example will check that `test/present_file` will be present. It will
not check the contents.

```toml
["test/present_file".file_present]
```

## Key Absent

`key_absent` will check if the key is not present in the file.

The next example will check that `test/present_file` has no key named `key`.

```toml
["test/present.toml".key_absent.key]
```

The key can be nested. In the next case it is sufficient that `key` is not present.
`super_key` may be present or absent.

```toml
["test/present.toml".key_absent.super_key.key]
```

This checker type can handle different kind of [mapping file types](#mapping-file-types)

## Key Value Present

`key_value_present` will check that the keys specified are present with the specified values.
Keys may be nested. Intermediate keys has to have mappings as values. When intermediate values
are not present, they will be added.

```toml
["test/present.toml".key_value_present]
key1 = 1
key2 = "value"
```

```toml
["test/present.toml".key_value_present.super_key]
key1 = 1
key2 = "value"
```

This checker type can handle different kind of [mapping file types](#mapping-file-types)

## Entry Absent

`entry_absent` will check that specified `__items__` are not present on the specified path.

```toml
["test/present.toml".entry_absent.key]
__items__ = [1, 2]
```

## Entry Present

`entry_present` will check that specified `__items__` are not present on the specified path.

```toml
["test/present.toml".entry_present.key]
__items__ = [1, 2]
```

## Key Value Regex Match

`key_value_regex_match` will check that the keys specified are present and the value matches the specified regex.
Of course, the regex can only match string values.
Keys may be nested. Intermediate keys has to have mappings as values. When intermediate values
are not present, they will be added.

```toml
["test/present.toml".key_value_regex_match]
key = 'v.*'
```

```toml
["test/present.toml".key_value_regex_match.super_key]
key = '[0-9]*'
```

Note: specify the regex as a raw string (single quotes) to be prevent escaping.

This checker type can handle different kind of [mapping file types](#mapping-file-types)

## Lines Absent

`lines_absent` will check that the file does not contain the lines as specified.

```toml
["test/present.txt".lines_absent]
__lines__ = """\
multi
line"""
```

```toml
["test/present.txt".lines_absent]
__lines__ = """single line"""
```

## Lines Present

`lines_present` will check that the file does not contain the lines as specified.

```toml
["test/present.txt".lines_present]
__lines__ = """\
multi
line"""
```

```toml
["test/present.txt".lines_present]
__lines__ = """single line"""
```

## Mapping File Types

The checker types with a key (key_absent, key_value_present, key_value_regex_match) can we used on several file types
which contains mappings:

| type | extension |
|------|-----------|
| toml | toml      |
| yaml | yaml, yml |
| json | json      |

The filetype will be determined by the extension. You can override this by specifying the filetype:

```toml
["test/present.toml".key_value_present]
__filetype__ = "json"
key1 = 1
key2 = "value"
```
