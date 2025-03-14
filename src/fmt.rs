mod error;
mod fmt_ast;
mod formatter;
mod raw_ast;

use crate::error::print_error;
use crate::formatter::{FormatStyle, Formatter};
use crate::raw_ast::parse_ast;
use clap::{Arg, command};
use pest::Parser;
use pest_derive::Parser;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, fs};

#[derive(Parser)]
#[grammar = "lc3.pest"]
struct LC3Parser;

static FORMATTED_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main() -> anyhow::Result<()> {
    let (style, check_mode, file_path) = get_from_cli();
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
                format_file(&style, path, content.as_str());
            }
            Err(err) => {
                eprintln!("{err}");
            }
        });

    if check_mode {
        todo!();
        // todo compare the diff
    }

    let count = FORMATTED_COUNT.load(Ordering::Relaxed);
    println!(
        "Formatted {} file{}.",
        count,
        (count > 1).then_some("s").unwrap_or("")
    );

    Ok(())
}

fn format_file(style: &FormatStyle, filename: &Path, file_content: &str) {
    match LC3Parser::parse(Rule::Program, file_content) {
        Ok(pairs) => {
            let program = parse_ast(pairs.into_iter().next().unwrap());
            let program = fmt_ast::StandardTransform::new(true, file_content).transform(program);
            let mut formatter = Formatter::new(style);
            formatter.format(program);
            // write back to the files
            match fs::write(filename, formatter.contents()) {
                Ok(_) => {
                    FORMATTED_COUNT.fetch_add(1, Ordering::Relaxed);
                    println!("Formatted {}.", filename.display());
                }
                Err(err) => {
                    eprintln!(
                        "Failed to write file {}, because {err}.",
                        filename.display()
                    );
                }
            }
        }
        Err(e) => print_error(
            filename.to_string_lossy().into_owned().as_str(),
            file_content,
            e,
        ),
    }
}

const DEFAULT_STYLE: FormatStyle = FormatStyle {
    directive_indent: 3,
    instruction_indent: 4,
    label_indent: 0,
    min_comment_distance_from_block: 1,
    space_block_to_comment: 1,
    space_comment_stick_to_body: 0,
    space_from_label_block: 1,
    space_from_start_end_block: 1,
};

const CONFIG_FILENAME: &str = "lc3-format.toml";
const CONFIG_FILENAME_EXTENSION: &str = "asm";

fn get_from_cli() -> (FormatStyle, bool, Vec<PathBuf>) {
    // lc3-fmt Options: --check -> i32 (0 formatted correctly, 1 diff and prints diff), optional
    // lc3-fmt Input: <file> . for all the files, or path to one file
    let matches = command!()
        .arg(
            Arg::new("check")
                .short('c')
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
        .get_matches();

    let style;
    let check_mode = matches.get_flag("check");
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

    let file_path: Vec<PathBuf> = match file_path.is_dir() {
        true => match fs::read_dir(file_path) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension()
                        .map_or(false, |ext| ext == CONFIG_FILENAME_EXTENSION)
                }) // Filter by .asm extension
                .collect(),
            Err(err) => {
                eprintln!("{err}");
                exit(1);
            }
        },
        false => {
            if file_path
                .extension()
                .map_or(false, |ext| ext == CONFIG_FILENAME_EXTENSION)
            {
                vec![file_path.to_path_buf()]
            } else {
                vec![]
            }
        }
    };

    style = match env::current_dir() {
        Ok(dir) => {
            let path = dir.join(CONFIG_FILENAME);
            match fs::read_to_string(&path) {
                Ok(content) => toml::from_str(&content).unwrap_or_else(|err| {
                    eprintln!(
                        "Cannot open {}! {}, fallback to the default settings",
                        CONFIG_FILENAME, err
                    );
                    DEFAULT_STYLE
                }),
                Err(err) => {
                    eprintln!(
                        "Cannot open {}! {}, fallback to the default settings",
                        CONFIG_FILENAME, err
                    );
                    DEFAULT_STYLE
                }
            }
        }
        Err(_) => {
            // fallback
            DEFAULT_STYLE
        }
    };

    (style, check_mode, file_path)
}
