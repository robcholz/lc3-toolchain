# lc3-toolchain

![Version](https://img.shields.io/badge/version-0.3.2-blue) ![Edition](https://img.shields.io/badge/edition-2024-orange)

Fast LC-3 assembly formatter + linter built for ECE109 but ready for any LC-3 codebase. Ships two binaries: `lc3fmt` (
formatter) and `lc3lint` (style linter) with friendly diagnostics.

<div style="text-align: center;">
    <img src="doc/check_mode.png" alt="Description" width="500">
</div>

## Features

- LC-3–aware parser that understands labels, directives, comments, and spacing rules
- Opinionated formatting with diff-able `--check` mode for CI/pre-commit
- Style linting for label/instruction/directive casing and colon rules
- Configurable via TOML; auto-discovers configs up the directory tree
- Inline diffs for formatting and codespan-highlighted lint diagnostics

## Quick Start

```bash
# Install (requires Rust toolchain)
cargo install lc3-toolchain

# Format a file or directory
lc3fmt path/to/file.asm
lc3fmt --check src/         # exits 1 if reformatting is needed

# Lint for style issues
lc3lint path/to/file.asm
```

## Install

- From crates.io: `cargo install lc3-toolchain`
- Or download a prebuilt binary from the Releases page and place it on your `PATH`
- Requires Rust 1.79+ (edition 2024). Tested on macOS and Linux.

## lc3fmt (formatter)

- Usage: `lc3fmt <file_or_directory> [--check] [--config-path <path>] [--print-config] [--verbose]`
- Exit codes: `0` success/no diff (check mode) • `1` reformat needed or I/O error
- Config discovery: looks for `lc3-format.toml` starting at `--config-path` (or CWD) and walking parents

**Before/after**

```asm
; before
LOOP    ADD R1,R1,#1    ;inc
        BRnzp LOOP

; after (default style)
LOOP        ADD     R1, R1, #1 ; inc
            BRnzp   LOOP
```

**Config sample (`lc3-format.toml`)**

```toml
[format-style]
indent-directive = 3
indent-instruction = 4
indent-label = 0
indent-min-comment-from-block = 1
space-block-to-comment = 1
space-comment-stick-to-body = 0
space-from-label-block = 1
space-from-start-end-block = 1
colon-after-label = true
fixed-body-comment-indent = false
directive-label-wrap = false
```

- `--print-config` dumps the effective defaults (see `doc/print_config.png`).

## lc3lint (style linter)

- Usage: `lc3lint <file_or_directory> [--config-path <path>] [--print-config] [--verbose]`
- Exit codes: `0` clean • `1` style violations or parse errors
- Rules: casing for labels/instructions/directives and whether labels must end with `:`

**Config sample (`lc3-lint.toml`)**

```toml
[lint-style]
colon-after-label = false
label-style = "ScreamingSnakeCase"
instruction-style = "ScreamingSnakeCase"
directive-style = "ScreamingSnakeCase"
```

## CI / Hooks

- Pre-commit: `lc3fmt --check .`
- CI: run both `lc3fmt --check .` and `lc3lint .` to fail on formatting or style drift.

## Contributing

Contributions are welcome—feel free to open an issue or submit a PR.

## License

GPL-3.0-only
