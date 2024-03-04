use clap::Parser;
use std::path::{Path, PathBuf};

use std::fs::{read_dir, File};
use std::io;

use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::io::Result;
use std::sync::Mutex;

use md5::Md5;
use sha2::{Digest, Sha256};

use console::{style, Term};
use indicatif::{DecimalBytes, ProgressBar, ProgressStyle};

use std::time::{Duration, Instant};

use regex::Regex;

use rayon::prelude::*;

/// A program to determine number of duplicate files (matching size and hashes) in a directory
#[derive(Parser)]
struct Args {
    /// Directory to scan for duplicates
    #[clap(default_value = "./")]
    directory: PathBuf,

    /// Recursively search directory
    #[clap(short, long, default_value_t = false)]
    recursive: bool,

    /// Exclude files and directories that begin with '.'
    #[clap(short = 'e', long, default_value_t = false)]
    exclude_dots: bool,

    /// Filter files by pattern, only files with names matching this pattern will be included
    #[clap(short = 'f', long)]
    filter: Option<Regex>,

    /// Follow symlinks, by default symbolic links are ignored
    #[clap(short = 'l', long, default_value_t = false)]
    follow_symlinks: bool,

    /// Use Md5 instead of Sha256, speeds up duplication detection but increases risk of collision drastically
    #[clap(short = '5', long, default_value_t = false)]
    md5: bool,

    /// Maximum file size allowed in bytes, larger files will be skipped
    #[clap(short = 'M', long)]
    max: Option<u64>,

    /// Minimum file size allowed in bytes, smaller files will be skipped
    #[clap(short, long)]
    min: Option<u64>,

    /// Hide progress information
    #[clap(short, long, default_value_t = false)]
    quiet: bool,

    /// Character to separate duplicate file paths with
    #[clap(short = '1', long, default_value = "\n")]
    separator: String,

    /// See total execution time of rupes
    #[clap(short, long, default_value_t = false)]
    time: bool,

    /// Display the amount of space wasted by each group of duplicate files
    #[clap(short, long, default_value_t = false)]
    size: bool,

    /// Display the total amount of space wasted by duplicate files
    #[clap(short = 'S', long, default_value_t = false)]
    total_size: bool,

    /// Display all details, equivalent of appending -sSt to command
    #[clap(short, long, default_value_t = false)]
    details: bool,

    /// Print rupes version
    #[clap(short = 'V', long, default_value_t = false)]
    version: bool,
}

#[derive(Debug)]
struct IdenticalFiles {
    paths: Vec<PathBuf>,
}

fn hash_file(path: &Path, args: &Args) -> Result<String> {
    let mut file = File::open(path)?;

    if args.md5 {
        let mut hasher = Md5::new();

        io::copy(&mut file, &mut hasher)?;

        let hash = hasher.finalize();
        let hash = base16ct::lower::encode_string(&hash);

        Ok(hash)
    } else {
        let mut hasher = Sha256::new();

        io::copy(&mut file, &mut hasher)?;

        let hash = hasher.finalize();
        let hash = base16ct::lower::encode_string(&hash);

        Ok(hash)
    }
}

fn handle_file(path: PathBuf, paths: &mut Vec<(u64, PathBuf)>, args: &Args) -> Result<()> {
    let metadata = path.metadata()?;
    let size = metadata.len();
    let file_type = metadata.file_type();
    let file_name = path.file_name().unwrap().to_string_lossy();

    // Guard against dot files/directories (if they are excluded)
    if args.exclude_dots && file_name.starts_with('.') {
        return Ok(());
    }

    // Handle files
    if file_type.is_file() {
        if let Some(filter) = args.filter.as_ref() {
            if !filter.is_match(&file_name) {
                return Ok(());
            }
        }

        if let Some(min) = args.min {
            if min > size {
                return Ok(());
            }
        }

        if let Some(max) = args.max {
            if max < size {
                return Ok(());
            }
        }

        paths.push((size, path));
        return Ok(());
    }

    if args.recursive && file_type.is_dir() {
        get_files(path, paths, args)?;
    }

    Ok(())
}

fn get_files(path: PathBuf, paths: &mut Vec<(u64, PathBuf)>, args: &Args) -> Result<()> {
    for entry in read_dir(path)? {
        let dir = entry?;
        let path = dir.path();

        if !args.follow_symlinks && dir.metadata()?.file_type().is_symlink() {
            continue;
        }

        handle_file(path, paths, args)?;
    }

    Ok(())
}

fn find_duplicates(
    paths: Vec<(u64, PathBuf)>,
    hashes_by_file_size: &mut HashMap<u64, HashMap<String, IdenticalFiles>>,
    progress: &ProgressBar,
    args: &Args,
) -> Result<()> {
    let hashes_by_file_size = Mutex::new(hashes_by_file_size);

    paths.par_iter().enumerate().for_each(|(_, (size, path))| {
        let hash = hash_file(path, args).unwrap();

        let mut hashes_by_file_size = hashes_by_file_size.lock().unwrap();
        let hashes = hashes_by_file_size.entry(*size).or_default();

        hashes
            .entry(hash)
            .and_modify(|identical_files| identical_files.paths.push(path.to_path_buf()))
            .or_insert(IdenticalFiles {
                paths: vec![path.to_path_buf()],
            });

        progress.inc(1);
    });

    Ok(())
}

fn scan_directory(
    hashes_by_file_size: &mut HashMap<u64, HashMap<String, IdenticalFiles>>,
    _term: &Term,
    args: &Args,
) -> Result<()> {
    if !args.directory.is_dir() {
        eprintln!("Please specify a valid directory to search");
        return Err(Error::new(ErrorKind::InvalidInput, "Please specify a valid directory to search"));
    }

    let get_files_spinner = if args.quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };
    get_files_spinner.enable_steady_tick(Duration::from_millis(100));
    get_files_spinner.set_style(ProgressStyle::with_template("{prefix} {spinner}").unwrap());
    get_files_spinner.set_prefix(format!("{} Scanning files", style("[1/2]").white()));

    let mut paths = Vec::new();
    get_files(args.directory.to_path_buf(), &mut paths, args)?;

    get_files_spinner.finish_and_clear();

    let progress = if args.quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(paths.len() as u64)
    };
    progress.set_style(
        ProgressStyle::with_template("{prefix} {pos:>7}/{len:7}\n[{bar:40.green/white}]")
            .unwrap()
            .progress_chars("=> "),
    );

    progress.set_prefix(format!("{} Finding duplicates", style("[2/2]").white()));

    find_duplicates(paths, hashes_by_file_size, &progress, args)?;

    progress.finish_and_clear();

    Ok(())
}

fn main() -> Result<()> {
    let now = Instant::now();
    let args = Args::parse();

    let term: Term = Term::stdout();

    if args.version {
        term.write_line(&format!("Rupes version {}", env!("CARGO_PKG_VERSION")))?;
        return Ok(());
    }

    let mut hashes_by_file_size = HashMap::new();
    scan_directory(&mut hashes_by_file_size, &term, &args)?;

    if hashes_by_file_size.is_empty() {
        term.write_line("No files to scan, rupes will now exit")?;
        return Ok(());
    }

    let term: Term = Term::buffered_stdout();

    // Final output

    let mut hashes_by_file_size: Vec<(u64, HashMap<String, IdenticalFiles>)> =
        hashes_by_file_size.into_iter().collect();
    hashes_by_file_size.sort_by_key(|pair| pair.0);

    term.write_line("")?;

    let mut total_size: u64 = 0;
    for (size, hashes) in hashes_by_file_size {
        for (_hash, mut identical_files) in hashes {
            if identical_files.paths.len() < 2 {
                continue;
            };

            let dupe_size = size * (identical_files.paths.len() - 1) as u64;
            total_size += dupe_size;

            identical_files.paths.sort();

            let paths: Vec<String> = identical_files
                .paths
                .iter()
                .map(|path_buf: &PathBuf| path_buf.display().to_string())
                .collect();

            let concatenated_paths = paths.join(&args.separator.to_string());

            term.write_line(&concatenated_paths)?;

            if args.size || args.details {
                term.write_line(&format!("^ {} of wasted space", DecimalBytes(dupe_size)))?;
            }
            term.write_line("")?
        }
    }

    if args.time || args.details {
        term.write_line(&format!("Took {:.2?} to complete", now.elapsed()))?;
    }
    if args.total_size || args.details {
        term.write_line(&format!("{} total wasted space", DecimalBytes(total_size)))?;
    }

    term.flush()?;

    Ok(())
}
