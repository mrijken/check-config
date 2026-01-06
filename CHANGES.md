# CHANGES

## 0.9.12

- Feat: add `always_copy_on_fix` flag to dir_copied

## 0.9.11

- Fix: fix typo in default filename

## 0.9.10

- Fix: fix checking the version for rust packages in package_present

## 0.9.9

- Feat: add source option to lines_present, so the lines can be loaded from a file

## 0.9.8

- Fix: replace variables in diff of file_copied

## 0.9.7

- Feat: show difference when source and destination are different for file_copied

## 0.9.6

- Feat: add check_config.toml to path when path is a directory
- Feat: use Cargo.toml (just like pyproject.toml) as possible source for the path

## 0.9.5

- Fix: fix using the version when installing a package via Cargo
- Fix: fix upgrade to latest version when version is not given

## 0.9.4

- Feat: add dir_absent
- Fix: make paths handling consistent

## 0.9.3

- Feat: add package_present and package_absent

## 0.9.2

- Feat: add variables and templating

## 0.9.1

- Feat: add dir_present

## 0.9.0

- BREAKING: refactor the check-config toml files. See documentation for new format.

## 0.8.6

- The path specified on the cli with, can also be a URL.

## 0.8.5

- Add tags to select checkers
- BREAKING: Remove `__config__` tag. Use `__include__` as top level key

## 0.8.4

- Improve readme

## 0.8.3

- Pass some cli options via env variables
- Add option to enable creation of intermediate directories

## 0.8.2

- Add marker to lines_present checktype
- Add marker to lines_absent checktype

## 0.8.1

- fix using `check-config` as command name
- fix usage of relative paths

## 0.8.0

- Add \_\_replacements_regex in lines_present
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
