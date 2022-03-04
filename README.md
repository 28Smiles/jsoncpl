# jsoncpl

Compare and Lint Json Files

```
USAGE:
    jsoncpl.exe [OPTIONS] [FOLDERS]...

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
