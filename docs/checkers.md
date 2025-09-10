# Checkers

`check-config` uses `checkers` which define the desired state of the configuration files. There are several
checker types (and more to come):

| checker type | description | fixable |
|------|-------------|---------|
| [file_absent](#file-absent) |  the file must be absent | yes |
| [file_present](#file-present) |  the file must be present, indifferent the content | yes |
| [file_regex_match](#file-regex-match) |  the content of the file must match the regex expression | when placeholder is present |
| [key_absent](#key-absent) | a specified key must be absent in a toml / yaml / json file  | yes |
| [key_value_present](#key-value-present) | a specified key with a specified value must be present in a toml / yaml / json file  | yes |
| [key_value_regex_match](#key-value-regex-match) | the value of a specified key must be match the specified regex in a toml / yaml / json file  | no |
| [entry_absent](#entry-absent) | a specified entry must be absent in the array of a toml / yaml / json file  | yes |
| [entry_present](#entry-present) | a specified entry  must be present in the of a toml / yaml / json file  | yes |
| [lines_absent](#lines-absent) | the specified lines must be absent | yes |
| [lines_present](#lines-present) | the specified lines must be present | yes |

## check-config.toml

The `check-config.toml` is the default entrypoint to define all checkers and configure check-config:

```toml
include = [  # optional list of toml files with additional checks
    "/home/me/.checkers/check.toml",  # absolute path
    "~/.checkers/check.toml",  # relative to home dir of current user
    "check.toml", # relative to this parent toml, indifferent if it is a filesystem path or a webserver path
    "py://my_package:checkers/python.toml", # path to file in python package
    "https//example.com/check.toml", # path on webserver
]
```

Note: When using a path to a Python package to include checkers, the activated Python (virtual) environment will be used.

And one or more checkers

```toml
[[<checker_name>]]
<checker object> = "<file_path>"
<checker specific key/values>
```

Note the double square brackets. We use an array of tables to define the checkers, so multiple
checkers of the same type may exist in the same toml file. If you use only one checker for
a certain type, you can also use single square brackets. However, to be consistent and
extensible, we advice to always use double brackets.

The syntax is slightly different per checker type. See the next sections for help about the checker definitions.

You can use arrays of toml tables when when a check has to be done more than once, ie:

```toml
[[lines_present]]
file = ".gitignore"
lines = "__pycache__"


[[lines_present]]
file = ".gitignore"
lines = ".cache"
```

## Tags

All checkers can have a `__tags__` key to make it possible to exclude or include this checker from the execution.
See [cli tags options](usage#Tags) for more information.

```toml
[[lines_present]]
file = ".gitignore"
tags = ["linux"]
lines = ".cache"
```

## File Absent

`file_absent` will check if the file is absent.

The next example will check that `test/absent_file` will be absent.

```toml
[[file_absent]]
file = "test/absent_file"
```

## File Present

`file_present` will check if the file is present.

The next example will check that `test/present_file` will be present. It will
not check the contents.

```toml
[[file_present]]
file = "test/present_file"
```

When the file does not exists, running with fix will create the file. At default an empty file
will be created.

This checker type can handle any text file.

This checker can also check for:
- placeholder
- regex
- permissions

### Placeholder

When a placeholder is given, the created file will contain the placeholder as content.

```toml
[[file_present]]
file = "test/present_file"
placeholder = "sample content"
```

### Regex Match

Checks whether the contents of the file matches the regex expression.

```toml
[[file_present]]
file = ".bashrc"
regex = 'export KEY=.*'
```

Multiple regex can be given when the file.checker_type pair is an array, ie:

```toml
[[file_present]]
file = ".bashrc"
regex = 'export KEY=.*'

[[file_present]]
file = ".bashrc"
regex = 'export ANOTHER_KEY=.*'
```

Note: specify the regex as a raw toml string (single quotes) to prevent escaping.

### Permissions

On Unix systems, you can check for the permissions:

```toml
[[file_present]]
file = ".bashrc"
permissions = "644"
```

The permissions need to be defined in the octal representation. See [chmod calculator](https://chmod-calculator.com/)
an explanation.

### Combinations

These extra checks can also be combined in one definition:

```toml
[[file_present]]
file = ".bashrc"
regex = 'export KEY=.*'
placeholder = "export KEY=hi"
permissions = "644"
```

## Key Absent

`key_absent` will check if the key is not present in the file.

The next example will check that `test/present_file` has no key named `key_to_be_absent`.

```toml
[[key_absent]]
file = "test/present.toml"
key.key_to_be_absent = {}
```

The key can be nested. In the next case it is sufficient that `key_to_be_absent` is not present.
`super_key` will not be removed if it contains also other keys.

```toml
[[key_absent]]
file = "test/present.toml"
key.super_key.key_to_be_absent = {}
```

This checker type can handle different kind of [mapping file types](#mapping-file-types)

## Key Value Present

`key_value_present` will check that the keys specified are present with the specified values.
Keys may be nested. Intermediate keys has to have mappings as values. When intermediate values
are not present, they will be added.

```toml
[[key_value_present]]
file = "test/present.toml"
key.key_to_add = 1
key.key_to_add_also = "value"
```

```toml
[key_value_present.super_key]
file = "test/present.toml"
key.super_key.key_to_add = {"inline_table" = "is also possible"}
```

This checker type can handle different kind of [mapping file types](#mapping-file-types)

## Entry Absent

`entry_absent` will check that all array items `entry.key<.key> = ["item"]` will be removed from the specified
file.

```toml
[[entry_absent]]
file = "test/present.toml"
key.list = [1, 2]
```

This checker type can handle different kind of [mapping file types](#mapping-file-types)
## Entry Present

`entry_present` will check that all array items `entry.key<.key> = ["item"]` will be added to the specified
file, if they do not exists already.

```toml
[[entry_present]]
file = "test/present.toml"
key.list = [1, 2]
```

This checker type can handle different kind of [mapping file types](#mapping-file-types)
## Key Value Regex Match

`key_value_regex_match` will check that the keys specified are present and the value matches the specified regex.
Of course, the regex can only match string values.
Keys may be nested. Intermediate keys has to have mappings as values. When intermediate values
are not present, they will be added.

```toml
[[key_value_regex_match]]
file = "test/present.toml"
key.key = 'v.*'
```

Note: specify the regex as a raw string (single quotes) to be prevent escaping.

This checker type can handle different kind of [mapping file types](#mapping-file-types)

## Lines Absent

`lines_absent` will check that the file does not contain the lines as specified.

```toml
[[lines_absent]]
file = "test/present.txt"
lines = """\
multi
line"""
```

```toml
[lines_absent]
file = "test/present.txt"
lines = """single line"""
```

You can also remove text between markers which removes the markers also

```toml
[[lines_absent]]
file = "test/present.txt"
marker = "# marker""
```

This will change the next text:

```text
Bla
# marker (check-config start)
Bla Bla
# marker (check-config end)
Bla
```

into

```text
Bla
Bla
```

## Lines Present

`lines_present` will check that the file does contain the lines as specified.

```toml
[[lines_present]]
file = "test/present.txt"
lines = """\
multi
line"""
```

```toml
["test/present.txt".lines_present]
file = "test/present.txt"
lines = """single line"""
```

Optionnally it can replace strings by regex, ie if you want to replace an export with a new value:

```toml
[[lines_present]]
file = "~/.bashrc"
lines = "export EDITOR=hx"
replacement_regex = "(?m)^export EDITOR=.*$"
```

Or you can use marker lines:


```toml
[[lines_present]]
file = "~/.bashrc"
lines = "export EDITOR=hx"
marker = "# marker"
```

Which replaces text from

```shell
alias ll='ls -alF'
# marker (check-config start)
export EDITOR=vi
# marker (check-config end)
```

into

```shell
alias ll='ls -alF'
# marker (check-config start)
export EDITOR=hx
# marker (check-config end)
```

When one of the markers is not present, the markers and the lines will be appended to the text.

Note: because the checkers are executed in sequence, one can add markers in one checker, which are replaced by
a next checker.

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
[[key_value_present]]
file = "test/present.toml" 
file_type = "json"
key.key = 1
```
