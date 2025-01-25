# YS 86 Formatter

## Requirement
- The Rust toolchain to compile and install the formatter

## Installation
```Shell
cargo install --git https://github.com/kensaa/y86fmt
```

## Usage
Like a normal formatter, it takes input from STDIN and output in STDOUT
### Exemple 
```Shell
cat source_code.ys | y86fmt > out.ys
```

## VSCode
If you want to be able to format a file using vscode, you will need to install a few extension on top of the formatter:
- [Y86 Syntax Highlighter](https://marketplace.visualstudio.com/items?itemName=abhinavk99.y86-vscode)
  - This is used to have syntax highlighting for .ys (y86) files and for VSCode to recognise .ys file as Y86
- [Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters)
  - You then need to add a bit of configuration to this extension for it to format Y86:
    - Open your VSCode config file (CTRL+SHIFT+P > "Preferences: Open User Settings (JSON)")
    - You then need to add the following to the end of the config file
        ```JSON
        "customLocalFormatters.formatters": [
            {
                "command": "y86fmt",
                "languages": ["y86"]
            }
        ],
         "[y86]": {
            "editor.defaultFormatter": "jkillian.custom-local-formatters"
        }
        ```
(One day I will create a VSCode extension managing all of that out of the box) (but I'm lazy)

## Backups
By default, the formatter create backup containing the source code before it was formatted, in case of a bug creating a loss of data (if that happen, create an issue so I can fix it please)
Those backups are located in the following directories
- For Windows : `%userprofile\AppData\Local\kensa\y86fmt\cache\backup\`
- For Linux : `~/.cache/y86fmt/backup/`

Backups can be disabled by passing the following flag when calling the formatter:
```Shell
y86fmt --disable-backup
```