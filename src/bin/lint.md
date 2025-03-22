# lc3lint

Static analyzer and linter for LC-3 assembly code.

## Usage

Basic usage:

```bash
lc3lint <file_or_directory>
```

This will analyze the specified LC-3 assembly file or all assembly files in the given directory for potential issues.

## Command-line Options

### `--config-path <path>`

Specifies a custom location for the configuration file. The tool will search for a `lc3-lint.toml` file starting from
this path and moving up through parent directories. The configuration file controls linting rules, severity levels, and
ignored warnings.

### `--print-config`

Outputs the current configuration settings to standard output. Helpful for creating your own custom configuration file
by using this output as a starting point.

### `--verbose`

Enables detailed output during the linting process. Shows information about each file being processed, and any issues
encountered.

## Error Categories

lc3lint detects several categories of potential issues:

- **Syntax errors**: Invalid instructions or directives
- **Style issues**: Inconsistent formatting or naming conventions

## Configuration

Create a `lc3-lint.toml` file to customize the linter's behavior:

```toml
[lint-style]
colon-after-label = false
label-style = "ScreamingSnakeCase"
instruction-style = "ScreamingSnakeCase"
directive-style = "ScreamingSnakeCase"
```

## Exit Codes

- **0**: No issues found
- **1**: Issues found
