use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};

pub fn get_relative_path(path: &Path) -> PathBuf {
    match env::current_dir() {
        Ok(root) => path.strip_prefix(root).unwrap_or(path),
        Err(_) => path,
    }
    .to_path_buf()
}

pub fn read_filepath(
    verbose_mode: bool,
    filename_extension: &str,
    filepath: PathBuf,
) -> Vec<PathBuf> {
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
                            if ext != filename_extension {
                                if verbose_mode {
                                    eprintln!(
                                        "Filename has to be {}, but found {}!",
                                        filename_extension,
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
                    if ext != filename_extension {
                        if verbose_mode {
                            eprintln!(
                                "Filename has to be .{}, but found .{}!",
                                filename_extension,
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
