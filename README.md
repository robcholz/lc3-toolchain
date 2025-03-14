# lc3fmt

![Version](https://img.shields.io/badge/version-0.1.1-blue)
![Edition](https://img.shields.io/badge/edition-2024-orange)

Format LC-3 Assembly Code, designed for ECE109 Spring 2025.

## Overview

`lc3fmt` is a code formatting tool specifically built for LC-3 assembly language. It provides automatic formatting to ensure consistent code style across your LC-3 assembly projects, making code more readable and maintainable.

## Installation

```bash
# Clone the repository
git clone https://github.com/finnsheng/lc3fmt.git

# Build the project
cd lc3fmt
cargo build --release

# Optional: Add to your PATH
```

## Usage

Basic usage:

```bash
lc3fmt <file_or_directory>
```

This will format the specified LC-3 assembly file or all assembly files in the given directory.

### Command-line Options

```
lc3fmt 0.1.1
Author: Finn Sheng@NCState Class of 2028
Format LC-3 Assembly Code, designed for ECE109 Spring 2025

USAGE:
    lc3fmt [OPTIONS] <file>

ARGS:
    <file>    Relative path to the file or directory containing the files to format

OPTIONS:
    -c, --check                Run in 'check' mode. Exits with 0 if input is formatted correctly.
                               Exits with 1 and prints a diff if formatting is required.
    --config-path <path>       Path for the rustfmt.toml configuration file. Recursively searches
                               the given path for the rustfmt.toml config file. If not found,
                               reverts to the input file path.
    --print-config             Dumps a default or minimal config to stdout
    --verbose                  Print verbose output
    -h, --help                 Print help information
```

### Examples

Format a single file:
```bash
lc3fmt program.asm
```

Check if a file is properly formatted without modifying it:
```bash
lc3fmt --check program.asm
```

Format all LC-3 assembly files in a directory:
```bash
lc3fmt ./src/
```

Print the default configuration:
```bash
lc3fmt --print-config
```

## Configuration

`lc3fmt` uses a configuration file similar to `rustfmt`. You can specify a custom configuration file using the `--config-path` option.

## For ECE109 Students

This tool was designed specifically for ECE109 (Spring 2025) at NC State to help you maintain clean and consistent LC-3 assembly code for your assignments and projects.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

GPL-v3

## Acknowledgments

- Thanks to the ECE109 Spring 2025 class at NC State University
- Inspired by `rustfmt` and other code formatting tools