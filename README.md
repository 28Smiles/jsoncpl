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
jsoncpl i18n/de i18n/en i18n/fr
```

Jsoncpl is highly configurable and provides autoformatting, for further explanation of the cli-parameters,
type `jsoncpl --help`:
```
USAGE:
    jsoncpl [OPTIONS] [FOLDERS]...

ARGS:
    <FOLDERS>...
            List the folders to search for files to lint and compare

OPTIONS:
        --format
            Whether the files should be automatically formatted

    -h, --help
            Print help information

        --indent <INDENT>
            The expected indentation of the json files
            
            [default: 4]

    -o, --order <ORDER>
            The expected sort order for keys in the json file
            
            [default: asc]
            [possible values: asc, desc]

    -s, --sort <SORT>
            The expected sorting algorithm for keys in the json file
            
            [default: default]
            [possible values: natural, default]

        --skip-check-order
            Whether the key order check should be skipped

        --skip-check-parity
            Whether the key parity check should be skipped

        --skip-check-style
            Whether the style check should be skipped

    -V, --version
            Print version information
```

## Installation

We provide binary releases for Linux, Windows and OSX.
