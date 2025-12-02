use clap::{Arg, command};
use lc3_toolchain::ast::get_ast;
use lc3_toolchain::ast::processed_ast::Program;
use lc3_toolchain::bin_utils;
use lc3_toolchain::bin_utils::get_relative_path;
use lc3_toolchain::error::print_error;
use lc3_toolchain::lint::{CaseStyle, Error, LintStyle, Linter};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{env, fs};

const CONFIG_FILENAME: &str = "lc3-lint.toml";
const CONFIG_FILENAME_EXTENSION: &str = "asm";

const BIN_NAME: &str = "lc3-toolchain lc3lint";
const ABOUT: &str = "Linter of LC3, designed for ECE109 Spring 2025. Exits with 0 if input has
                        correct style. Exits with 1 and prints a diff refactoring is required.";

static VERBOSE_MODE: AtomicBool = AtomicBool::new(false);

const DEFAULT_STYLE: LintStyle = LintStyle {
    colon_after_label: false,
    label_style: CaseStyle::ScreamingSnakeCase,
    instruction_style: CaseStyle::ScreamingSnakeCase,
    directive_style: CaseStyle::ScreamingSnakeCase,
};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ConfigLintStyle {
    colon_after_label: Option<bool>,
    label_style: Option<CaseStyle>,
    instruction_style: Option<CaseStyle>,
    directive_style: Option<CaseStyle>,
}

#[derive(Default, Serialize, Deserialize)]
struct Config {
    #[serde(rename = "lint-style")]
    lint_style: ConfigLintStyle,
}

fn main() {
    let matches = command!()
        .name(BIN_NAME)
        .about(ABOUT)
        .help_template(
            "{name} {version}\nAuthor: {author}\n{about}\n\n{usage-heading}\n{usage}\n\n{all-args}",
        )
        .arg(
            Arg::new("file")
                .help("Relative path to the file or directory containing the files to check")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("config-path")
                .long("config-path")
                .help(format!(
                    r#"Path for the configuration file. Recursively searches
                the given path for the {} config file. If not
                found, reverts to the input file path."#,
                    CONFIG_FILENAME
                ))
                .required(false),
        )
        .arg(
            Arg::new("print-config")
                .long("print-config")
                .help(r#"Dumps a default or minimal config to stdout"#)
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help(r#"Print verbose output"#)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    VERBOSE_MODE.store(matches.get_flag("verbose"), Ordering::Relaxed);
    let file_path = matches
        .get_one::<String>("file")
        .expect("File path is required");
    let file_path = match env::current_dir() {
        Ok(root) => root.join(file_path),
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };
    let file_path = bin_utils::read_filepath(
        VERBOSE_MODE.load(Ordering::Relaxed),
        CONFIG_FILENAME_EXTENSION,
        file_path,
    );
    let style = read_style(matches.get_one::<String>("config-path").map(PathBuf::from));

    if matches.get_flag("print-config") {
        print_style(&style);
    }

    let mut success = true;

    for path in file_path {
        match fs::read_to_string(&path) {
            Ok(content) => {
                let path_buf = get_relative_path(&path);
                let relative_path = path_buf.as_path();
                match check_syntax_error(relative_path, &content) {
                    None => {}
                    Some(program) => {
                        let results = Linter::new(style, program).check();
                        match results {
                            Ok(_) => {}
                            Err(errors) => {
                                // Visualize errors using codespan-reporting
                                let mut files = codespan_reporting::files::SimpleFiles::new();
                                let file_id = files.add(relative_path.to_string_lossy(), &content);

                                let config = codespan_reporting::term::Config::default();
                                let writer =
                                    codespan_reporting::term::termcolor::StandardStream::stderr(
                                        codespan_reporting::term::termcolor::ColorChoice::Auto,
                                    );

                                for error in errors {
                                    let diagnostic = create_diagnostic_from_error(&error, file_id);
                                    codespan_reporting::term::emit(
                                        &mut writer.lock(),
                                        &config,
                                        &files,
                                        &diagnostic,
                                    )
                                    .expect("Failed to emit diagnostic");
                                }

                                success = false;
                            }
                        }
                    }
                }
            }
            Err(err) => {
                if VERBOSE_MODE.load(Ordering::Relaxed) {
                    eprintln!("{err}");
                }
            }
        }
    }

    if !success {
        exit(1);
    }
}

// print or return ast
fn check_syntax_error(filename: &Path, file_content: &str) -> Option<Program> {
    match get_ast(file_content) {
        Ok(program) => Some(program),
        Err(e) => {
            print_error(
                filename.to_string_lossy().into_owned().as_str(),
                file_content,
                *e,
            );
            None
        }
    }
}

fn read_style(filepath_opt: Option<PathBuf>) -> LintStyle {
    let filepath: Option<PathBuf> = match filepath_opt.as_ref() {
        // read the current one
        None => match env::current_dir() {
            Ok(dir) => Some(dir.join(CONFIG_FILENAME)),
            Err(_) => None,
        },
        Some(path) => Some(path.clone()),
    };

    if filepath.is_none() {
        return DEFAULT_STYLE;
    }

    let path = filepath.as_ref().unwrap();

    match fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => config_lint_style_to_lint_style(&DEFAULT_STYLE, config.lint_style),
            Err(err) => {
                eprintln!(
                    "Cannot parse {}! {}, fallback to the default settings",
                    CONFIG_FILENAME, err
                );
                DEFAULT_STYLE
            }
        },
        Err(err) => {
            if filepath_opt.is_some() {
                eprintln!(
                    "Cannot open {}! {}, fallback to the default settings",
                    CONFIG_FILENAME, err
                );
            }
            DEFAULT_STYLE
        }
    }
}

fn config_lint_style_to_lint_style(
    default: &LintStyle,
    config_lint_style: ConfigLintStyle,
) -> LintStyle {
    LintStyle {
        colon_after_label: config_lint_style
            .colon_after_label
            .unwrap_or(default.colon_after_label),
        label_style: config_lint_style.label_style.unwrap_or(default.label_style),
        instruction_style: config_lint_style
            .instruction_style
            .unwrap_or(default.instruction_style),
        directive_style: config_lint_style
            .directive_style
            .unwrap_or(default.directive_style),
    }
}

fn print_style(style: &LintStyle) {
    let toml_str = toml::to_string(style).expect("Failed to serialize FormatStyle to TOML");
    println!("{toml_str}");
}

fn create_diagnostic_from_error(
    error: &Error,
    file_id: usize,
) -> codespan_reporting::diagnostic::Diagnostic<usize> {
    use codespan_reporting::diagnostic::{Diagnostic, Label};

    // Determine error message based on error type
    let message = match (error.case_style_error(), error.colon_style_error()) {
        (Err((expected, found)), _) => match found {
            Some(found_style) => format!(
                "Invalid case style: found {:?}, expected {:?}",
                found_style, expected
            ),
            None => format!("Unknown case style, expected {:?}", expected),
        },
        (_, Err(_)) => "Invalid colon style".to_string(),
        _ => "Unknown error".to_string(),
    };

    // Create the diagnostic with appropriate severity
    Diagnostic::warning()
        .with_message(message)
        .with_labels(vec![
            Label::primary(file_id, *error.span().start()..*error.span().end())
                .with_message("Warning occurred here"),
        ])
        .with_notes(vec![
            "See the style guide for more information on formatting rules.".to_string(),
        ])
}
