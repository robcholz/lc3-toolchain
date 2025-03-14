# lc3-toolchain

![Version](https://img.shields.io/badge/version-0.1.1-blue)
![Edition](https://img.shields.io/badge/edition-2024-orange)

LC-3 Assembly Toolchain, designed for ECE109 Spring 2025.

## Overview

`lc3-toolchain` contains a code formatting tool specifically built for LC-3 assembly language.
It provides automatic formatting to ensure consistent code style across your LC-3 assembly projects, making code more
readable and maintainable.

## Installation

Download from release.

Or:
Cargo

```bash
cargo install lc3-toolchain
```

## Usage

Basic usage:

```bash
lc3fmt <file_or_directory>
```

This will format the specified LC-3 assembly file or all assembly files in the given directory.

### Command-line Options

#### `-c, --check`

Run in 'check' mode. Exits with 0 if input is formatted correctly.
Exits with 1 and prints a diff if formatting is required.

Validates files for proper formatting without making changes.
Useful for CI/CD pipelines or pre-commit hooks to ensure code style compliance.
Returns a non-zero exit code if any files need formatting.

<img src="doc/check_mode.png" alt="Description" width="500">

#### `--config-path <path>`

Specifies a custom location for the configuration file. The tool will search for a `lc3-format.toml` file starting from
this path and moving up through parent directories. The configuration file controls formatting rules like indentation
style, comment alignment, and label positioning.

#### `--print-config`

Outputs the current configuration settings to standard output. Helpful for creating your own custom configuration file
by using this output as a starting point.

<img src="doc/print_config.png" alt="Description" width="500">

#### `--verbose`

Enables detailed output during the formatting process. Shows information about each file being processed, and any issues
encountered.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

GPL-v3
