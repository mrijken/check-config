# Glossary

| item           | definition                                                                                        |
| -------------- | ------------------------------------------------------------------------------------------------- |
| checker type   | A type of the check to be executed. Like `lines_present`                                          |
| checker object | The object being checked, like a file                                                             |
| checker        | A concrete instance of a check to be executed, like an `lines_present` for the object `~/.bashrc` |
| check          | A concrete execution of a checker                                                                 |

## Naming conventions

The name of a checker type consist of `<noun>_<adjective>`.

So ie `file_present` can be read as `file is present` and grammatical analysed as:

- `file` – noun (subject of the sentence).
- `is` – verb (linking verb, present tense, 3rd person singular of to be).
- `present` – adjective (subject complement, describing the state of the file).
