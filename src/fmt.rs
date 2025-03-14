mod error;
mod fmt_ast;
mod formatter;
mod raw_ast;

use crate::error::print_error;
use crate::formatter::{FormatStyle, Formatter};
use crate::raw_ast::parse_ast;
use clap::{Arg, command};
use console::{Style, style};
use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{env, fmt, fs};

#[derive(Parser)]
#[grammar = "lc3.pest"]
struct LC3Parser;

static FORMATTED_COUNT: AtomicUsize = AtomicUsize::new(0);
static FILE_DIFF_COUNT: AtomicUsize = AtomicUsize::new(0);
static VERBOSE_MODE: AtomicBool = AtomicBool::new(false);
static CHECK_MODE: AtomicBool = AtomicBool::new(false);

fn main() -> anyhow::Result<()> {
    let (style, file_path) = get_from_cli();
    file_path
        .iter()
        .for_each(|path| match fs::read_to_string(path) {
            Ok(content) => {
                let path: &Path = match env::current_dir() {
                    Ok(root) => match path.strip_prefix(root) {
                        Ok(relative_path) => relative_path,
                        Err(_) => path,
                    },
                    Err(_) => path,
                };
                let result = format_file(&style, path, content.as_str());
                match result {
                    None => {}
                    Some(formatter) => {
                        if CHECK_MODE.load(Ordering::Relaxed) {
                            if check_file_diff(path, content.as_str(), &formatter) {
                                FILE_DIFF_COUNT.fetch_add(1, Ordering::Relaxed);
                            }
                        } else {
                            write_file(path, &formatter);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("{err}");
            }
        });

    let count = FORMATTED_COUNT.load(Ordering::Relaxed);
    if !CHECK_MODE.load(Ordering::Relaxed) {
        println!(
            "Formatted {} file{}.",
            count,
            (count > 1).then_some("s").unwrap_or("")
        );
    }

    if FILE_DIFF_COUNT.load(Ordering::Relaxed) > 0 {
        exit(1);
    }

    Ok(())
}

fn format_file<'a>(
    style: &'a FormatStyle,
    filename: &Path,
    file_content: &str,
) -> Option<Formatter<'a>> {
    match LC3Parser::parse(Rule::Program, file_content) {
        Ok(pairs) => {
            let program = parse_ast(pairs.into_iter().next().unwrap());
            let program = fmt_ast::StandardTransform::new(true, file_content).transform(program);
            let mut formatter = Formatter::new(style);
            formatter.format(program);
            Some(formatter)
        }
        Err(e) => {
            print_error(
                filename.to_string_lossy().into_owned().as_str(),
                file_content,
                e,
            );
            None
        }
    }
}

fn write_file(filename: &Path, formatter: &Formatter) {
    // write back to the files
    match fs::write(filename, formatter.contents()) {
        Ok(_) => {
            FORMATTED_COUNT.fetch_add(1, Ordering::Relaxed);
            if VERBOSE_MODE.load(Ordering::Relaxed) {
                println!("Formatted {}.", filename.display());
            }
        }
        Err(err) => {
            eprintln!(
                "Failed to write file {}, because {err}.",
                filename.display()
            );
        }
    }
}

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

fn check_file_diff(filename: &Path, file_content: &str, formatter: &Formatter) -> bool {
    let formatted = String::from_utf8_lossy(formatter.contents());
    let diff = TextDiff::configure()
        .algorithm(similar::Algorithm::Patience)
        .diff_lines(formatted.as_ref(), file_content);

    let is_diff = diff.iter_all_changes().next().is_some() && (formatted != file_content);

    if is_diff {
        println!("File differs: {}", filename.display());
        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                println!("{:-^1$}", "-", 80);
            }
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let (sign, s) = match change.tag() {
                        ChangeTag::Delete => ("-", Style::new().red()),
                        ChangeTag::Insert => ("+", Style::new().green()),
                        ChangeTag::Equal => (" ", Style::new().dim()),
                    };
                    print!(
                        "{}{} |{}",
                        style(Line(change.old_index())).dim(),
                        style(Line(change.new_index())).dim(),
                        s.apply_to(sign).bold(),
                    );
                    for (emphasized, value) in change.iter_strings_lossy() {
                        if emphasized {
                            print!("{}", s.apply_to(value).underlined().on_black());
                        } else {
                            print!("{}", s.apply_to(value));
                        }
                    }
                    if change.missing_newline() {
                        println!();
                    }
                }
            }
        }
    }

    is_diff
}

const DEFAULT_STYLE: FormatStyle = FormatStyle {
    indent_directive: 3,
    indent_instruction: 4,
    indent_label: 0,
    indent_min_comment_from_block: 1,
    space_block_to_comment: 1,
    space_comment_stick_to_body: 0,
    space_from_label_block: 1,
    space_from_start_end_block: 1,
};

const CONFIG_FILENAME: &str = "lc3-format.toml";
const CONFIG_FILENAME_EXTENSION: &str = "asm";

fn get_from_cli() -> (FormatStyle, Vec<PathBuf>) {
    let matches = command!()
        .help_template(
            "{name} {version}\nAuthor: {author}\n{about}\n\n{usage-heading}\n{usage}\n\n{all-args}",
        )
        .arg(
            Arg::new("check")
                .short('c')
                .long("check")
                .help(
                    "Run in 'check' mode. Exits with 0 if input is
                        formatted correctly. Exits with 1 and prints a diff if
                        formatting is required.",
                )
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file")
                .help("Relative path to the file or directory containing the files to format")
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
    CHECK_MODE.store(matches.get_flag("check"), Ordering::Relaxed);
    let style = read_style(
        matches
            .get_one::<String>("config-path")
            .map_or(None, |s| Some(PathBuf::from(s))),
    );
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
    let file_path = read_filepath(file_path);

    if matches.get_flag("print-config") {
        print_style(&style);
    }

    (style, file_path)
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "format-style")]
    pub format_style: ConfigFormatStyle,
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFormatStyle {
    pub indent_directive: Option<u8>,
    pub indent_instruction: Option<u8>,
    pub indent_label: Option<u8>,
    pub indent_min_comment_from_block: Option<u8>,
    pub space_block_to_comment: Option<u8>,
    pub space_comment_stick_to_body: Option<u8>,
    pub space_from_label_block: Option<u8>,
    pub space_from_start_end_block: Option<u8>,
}

fn read_style(filepath_opt: Option<PathBuf>) -> FormatStyle {
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

    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<Config>(&content) {
            Ok(config) => config_format_style_to_format_style(&DEFAULT_STYLE, config.format_style),
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

fn read_filepath(filepath: PathBuf) -> Vec<PathBuf> {
    match filepath.is_dir() {
        true => match fs::read_dir(filepath) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    let ext = path.extension();
                    match ext {
                        None => false,
                        Some(ext) => {
                            if ext != CONFIG_FILENAME_EXTENSION {
                                if VERBOSE_MODE.load(Ordering::Relaxed) {
                                    eprintln!(
                                        "Filename has to be {}, but found {}!",
                                        CONFIG_FILENAME_EXTENSION,
                                        ext.to_string_lossy().as_ref()
                                    );
                                }
                                false
                            } else {
                                true
                            }
                        }
                    }
                }) // Filter by .asm extension
                .collect(),
            Err(err) => {
                eprintln!("{err}");
                exit(1);
            }
        },
        false => {
            let extension = filepath.extension();
            match extension {
                None => {
                    vec![]
                }
                Some(ext) => {
                    if ext != CONFIG_FILENAME_EXTENSION {
                        if VERBOSE_MODE.load(Ordering::Relaxed) {
                            eprintln!(
                                "Filename has to be .{}, but found .{}!",
                                CONFIG_FILENAME_EXTENSION,
                                ext.to_string_lossy().as_ref()
                            );
                        }
                        vec![]
                    } else {
                        vec![filepath]
                    }
                }
            }
        }
    }
}

fn config_format_style_to_format_style(
    default: &FormatStyle,
    config_format_style: ConfigFormatStyle,
) -> FormatStyle {
    FormatStyle {
        indent_directive: config_format_style
            .indent_directive
            .unwrap_or(default.indent_directive),
        indent_instruction: config_format_style
            .indent_instruction
            .unwrap_or(default.indent_instruction),
        indent_label: config_format_style
            .indent_label
            .unwrap_or(default.indent_label),
        indent_min_comment_from_block: config_format_style
            .indent_min_comment_from_block
            .unwrap_or(default.indent_min_comment_from_block),
        space_block_to_comment: config_format_style
            .space_block_to_comment
            .unwrap_or(default.space_block_to_comment),
        space_comment_stick_to_body: config_format_style
            .space_comment_stick_to_body
            .unwrap_or(default.space_comment_stick_to_body),
        space_from_label_block: config_format_style
            .space_from_label_block
            .unwrap_or(default.space_from_label_block),
        space_from_start_end_block: config_format_style
            .space_from_start_end_block
            .unwrap_or(default.space_from_start_end_block),
    }
}

fn print_style(style: &FormatStyle) {
    let toml_str = toml::to_string(style).expect("Failed to serialize FormatStyle to TOML");
    println!("{toml_str}");
}
