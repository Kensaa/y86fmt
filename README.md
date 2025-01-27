# Y86 Formatter

## Requirements
- The Rust toolchain is required to compile and install the formatter.

## Installation
You can install y86fmt using cargo:
```bash
cargo install --git https://github.com/kensaa/y86fmt
```
Or you can install it manually by downloading the executable from the [Releases](https://github.com/Kensaa/y86fmt/releases/latest)

## Usage
Like a normal formatter, it takes input from `STDIN` and outputs to `STDOUT`.

### Example
```bash
cat source_code.ys | y86fmt > out.ys
```

## VSCode Integration
If you'd like to format `.ys` files in VSCode, you'll need to install a few extensions in addition to the formatter:

- [Y86 Syntax Highlighter](https://marketplace.visualstudio.com/items?itemName=abhinavk99.y86-vscode)  
  This extension provides syntax highlighting for `.ys` (Y86) files and enables VSCode to recognize `.ys` files as Y86.

- [Custom Local Formatters](https://marketplace.visualstudio.com/items?itemName=jkillian.custom-local-formatters)  
  You will need to configure this extension to format Y86 files:
  
  1. Open your VSCode configuration file:  
     `CTRL+SHIFT+P > "Preferences: Open User Settings (JSON)"`
  2. Add the following configuration to the end of the file:
     ```json
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
     
     * Note: if you downloaded the formatter using the releases, you need to change the command if the config to add the path to the file, or add the formatter to the $PATH environment variable

*(One day, I plan to create a dedicated VSCode extension to handle this out of the box—but I’m lazy for now.)*

## Backups
By default, the formatter creates backups of the source code before formatting, to prevent data loss in case of bugs. If an issue arises, please create a GitHub issue so I can address it.

Backup locations:  
- **Windows**: `%userprofile%\AppData\Local\kensa\y86fmt\cache\backup\`  
- **Linux**: `~/.cache/y86fmt/backup/`

You can disable backups by passing the following flag when calling the formatter:
```bash
y86fmt --disable-backup
```

## Disclaimer
This formatter was coded during a single-night, caffeine-fueled frenzy. As a result, it is almost certainly rough around the edges because I don’t fully know what I’m doing (hence the backups). I haven’t extensively used it yet, so there are likely several bugs. If you encounter any issues, please let me know so I can try to fix them.
