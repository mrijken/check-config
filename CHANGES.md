# CHANGES

## 0.8.2

- Add marker to lines_present checktype
- Add marker to lines_absent checktype

## 0.8.1

- fix using `check-config` as command name
- fix usage of relative paths

## 0.8.0

- Add __replacements_regex in lines_present
- BREAKING: use [__config__] in stead of [check-config] as config table
- BREAKING: use `check-config.toml` as default name
- preserve formatting of toml files, including comments
- fix several small bugs and command output
- fix usage of relative urls and home dir (ie " ~/.bashrc")
- add list-checkers option

## 0.7.1

- fix reading python paths

## 0.7.0

- fix rename additional checks to include [#7](https://github.com/mrijken/check-config/issues/7)
- fix relative includes [#8](https://github.com/mrijken/check-config/issues/8)
- add fallback to pyproject.toml [[#6](https://github.com/mrijken/check-config/issues/6)]

## 0.6.1

- BREAKING: Use check_config.toml (in stead of checkers.toml) as default style file
- Add pre-commit hook

## 0.6.0

- Add optional placeholder when creating a file with file_present
- Add optional placeholder when creating a file with file_regex

## 0.5.1

- Update documentation

## 0.5.0

- Support http(s) location for checkers.

## 0.4.1

- Update dependencies

## 0.4.0

- Refactor checkers
- Add file regex

## 0.3.3

- Add documentation

## 0.3.2

- Fix: improve error handling

## 0.3.1

- Fix: empty json and yaml files where not processed correctly

## 0.3.0

- Support files from Python packages.

## 0.2.0

- Add entry_absent and entry_present

## 0.1.1

- Fix stable classifier

## 0.1.0

- initial release
