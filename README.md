[![Main](https://github.com/28Smiles/jsoncpl/actions/workflows/test.yml/badge.svg)](https://github.com/28Smiles/jsoncpl/actions/workflows/test.yml)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Latest Stable](https://img.shields.io/github/v/release/28Smiles/jsoncpl?label=latest%20stable)](https://github.com/28Smiles/jsoncpl/releases/latest)
[![Latest Release](https://img.shields.io/github/v/release/28Smiles/jsoncpl?include_prereleases&label=latest%20release)](https://github.com/28Smiles/jsoncpl/releases)

# Jsoncpl

Jsoncpl is a json comparison and linting tool. It was designed to lint dictionary-like json files like the language files
from [angular-l10n](https://github.com/robisim74/angular-l10n), 
therefore currently its parser only supports json-objects and strings. Jsoncpl compares the file hierarchy,
the key order and compares the keys of every file with all its counterparts (same relative folder and filename).

For example, if you have a file structure like this:
```
i18n:
  - de:
    - common.json
    - main:
      - editor.json
  - en:
    - common.json
    - main:
      - editor.json
  - fr:
    - common.json
    - main:
      - editor.json
```
then you would need to provide the paths to the folders `de`, `en` and `fr` via the cli:
```
jsoncpl lint i18n/de i18n/en i18n/fr
```

Jsoncpl is highly configurable and provides autoformatting, for further explanation of the cli-parameters,
type `jsoncpl --help`:
```
A tool for linting and formatting json files

Usage: jsoncpl.exe [OPTIONS] <COMMAND>

Commands:
  format
          Format the provided files according to the style parameters
  lint
          Check the provided files according to the style parameters
  help
          Print this message or the help of the given subcommand(s)

Options:
  -a, --algorithm <ALGORITHM>
          The expected sorting algorithm for keys in the json file
          
          [default: default]

          Possible values:
          - natural: Sort the keys by natural sort
          - default: Sort the keys by classical sort
          - ignore:  Ignore sort order

  -o, --order <ORDER>
          The expected sort order for keys in the json file
          
          [default: asc]

          Possible values:
          - asc:  Sort the keys by ascending order
          - desc: Sort the keys by descending order

  -l, --line-endings <LINE_ENDINGS>
          The expected line endings of the json file
          
          [default: lf]

          Possible values:
          - crlf:   Add \\r\\n to the end of an entry
          - lf:     Add \\n to the end of an entry
          - none:   Add no linebreaks
          - any:    Accept any linebreak (\r\n or \n) (defaults to \n in formatting)
          - ignore: Accept anything (defaults to \n in formatting)

  -i, --indent <INDENT>
          The expected indentation of the json files
          
          [default: four]

          Possible values:
          - tab:    Indent with \t
          - two:    Indent with "  "
          - four:   Indent with "    "
          - ignore: Ignore indentation

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information
```

## Installation

We provide binary releases for Linux, Windows and OSX.
