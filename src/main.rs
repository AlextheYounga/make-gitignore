use clap::Parser;
use reqwest::blocking;
use std::collections::HashMap;
use std::env::current_dir;
use std::error::Error as StdError;
use std::fs::{self, File};
use std::io::{Result as IoResult, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

mod ui;

use ui::FetchStatus;

const GENERAL_IGNORE_LINES: &[&str] = &[
    ".DS_Store",
    "*DS_Store",
    "Thumbs.db",
    "Desktop.ini",
    "*.db",
    "*.sqlite",
    "*.sqlite3",
    "__trash__",
    ".Trashes",
    "*.log",
    "*.tmp",
    "*.temp",
    "*.bak",
    "*~",
    "*.swp",
    "*.swo",
];

const GITIGNORE_ARCHIVE_URL: &str = "https://github.com/github/gitignore/archive/refs/heads/main.zip";

type LanguageMap = HashMap<String, String>;

#[derive(Clone)]
struct CachePaths {
    app_cache_dir: PathBuf,
    extract_dir: PathBuf,
    zip_path: PathBuf,
    last_synced_path: PathBuf,
}

fn append_general_ignores(output_file: &mut File) -> IoResult<()> {
    writeln!(output_file)?;
    writeln!(output_file, "# General")?;

    for line in GENERAL_IGNORE_LINES {
        writeln!(output_file, "{line}")?;
    }

    Ok(())
}

#[derive(Parser)]
#[command(name = "gitignore")]
#[command(about = "Generate .gitignore files from templates", long_about = None)]
struct Cli {
    /// Comma-separated list of languages (e.g., --languages=Rust,Python,Node)
    #[arg(long, value_delimiter = ',')]
    languages: Option<Vec<String>>,
}

fn check_last_synced(last_synced_path: &PathBuf) -> bool {
    let timestamp_secs = fs::read_to_string(last_synced_path).ok().and_then(|s| s.trim().parse::<u64>().ok());
    let last_sync_time = timestamp_secs.map(|ts| UNIX_EPOCH + Duration::from_secs(ts));
    last_sync_time
        .and_then(|last| SystemTime::now().duration_since(last).ok())
        .is_some_and(|duration| duration.as_secs() < 24 * 3600)
}

fn cache_paths() -> Result<CachePaths, Box<dyn StdError>> {
    let cache_dir = dirs::cache_dir().ok_or("Could not find cache directory")?;
    let app_cache_dir = cache_dir.join("make-gitignore");
    let extract_dir = cache_dir.join("gitignore-templates");
    fs::create_dir_all(&app_cache_dir)?;

    Ok(CachePaths {
        zip_path: app_cache_dir.join("gitignore-main.zip"),
        last_synced_path: app_cache_dir.join("last_synced"),
        app_cache_dir,
        extract_dir,
    })
}

fn has_cached_templates(paths: &CachePaths) -> bool {
    paths.extract_dir.join("gitignore-main").is_dir()
}

fn download_gitignore_archive(zip_path: &Path) -> Result<(), Box<dyn StdError>> {
    let bytes = blocking::get(GITIGNORE_ARCHIVE_URL)?.bytes()?;
    let mut file = File::create(zip_path)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn replace_extracted_repo(zip_path: &Path, extract_dir: &Path) -> Result<(), Box<dyn StdError>> {
    let staging_dir = extract_dir.with_extension("next");
    let backup_dir = extract_dir.with_extension("old");

    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)?;
    }
    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)?;
    }

    unzip_gitignore_repo(zip_path, staging_dir.clone())?;

    if extract_dir.exists() {
        fs::rename(extract_dir, &backup_dir)?;
    }
    fs::rename(&staging_dir, extract_dir)?;

    if backup_dir.exists() {
        fs::remove_dir_all(&backup_dir)?;
    }

    Ok(())
}

fn refresh_gitignore_repo(paths: &CachePaths) -> Result<PathBuf, Box<dyn StdError>> {
    let temp_zip_path = paths.app_cache_dir.join("gitignore-main.zip.download");

    download_gitignore_archive(&temp_zip_path)?;
    replace_extracted_repo(&temp_zip_path, &paths.extract_dir)?;
    fs::rename(&temp_zip_path, &paths.zip_path)?;

    // Write last_synced timestamp (seconds since UNIX_EPOCH)
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    fs::write(&paths.last_synced_path, now_secs.to_string())?;

    Ok(paths.extract_dir.clone())
}

fn cache_gitignore_repo() -> Result<PathBuf, Box<dyn StdError>> {
    let paths = cache_paths()?;

    if has_cached_templates(&paths) && check_last_synced(&paths.last_synced_path) {
        println!("Using cached gitignore archive: {}", paths.extract_dir.display());
        return Ok(paths.extract_dir);
    }

    refresh_gitignore_repo(&paths)
}

fn unzip_gitignore_repo(zip_path: &Path, extract_dir: PathBuf) -> Result<PathBuf, Box<dyn StdError>> {
    // Remove old extraction if it exists
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir)?;
    }

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    archive.extract(&extract_dir)?;
    Ok(extract_dir)
}

fn scan_gitignore_templates(extract_dir: &Path) -> Result<LanguageMap, Box<dyn StdError>> {
    // The extracted directory will be gitignore-templates/gitignore-main/
    let repo_root = extract_dir.join("gitignore-main");

    let mut language_map = HashMap::new();

    // Read all entries in the root directory
    for entry in fs::read_dir(&repo_root)? {
        let entry = entry?;
        let path = entry.path();

        // Only process files (not directories)
        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if name.ends_with(".gitignore") => name,
            _ => continue,
        };

        // Extract language name (e.g., "Python" from "Python.gitignore")
        let Some(language) = filename.strip_suffix(".gitignore") else {
            continue;
        };
        let language = language.to_string();
        language_map.insert(language, fs::read_to_string(path)?);
    }

    println!("Found {} gitignore templates", language_map.len());
    Ok(language_map)
}

fn write_gitignore(
    selected_languages: &[String],
    language_map: &LanguageMap,
    output_path: &Path,
) -> Result<(), Box<dyn StdError>> {
    if selected_languages.is_empty() {
        return Err("No languages selected".into());
    }

    // Always create .gitignore by appending templates in order, preserving formatting.
    let mut output_file = File::create(output_path)?;

    for (idx, language) in selected_languages.iter().enumerate() {
        let contents = language_map.get(language).ok_or(format!("Language '{language}' not found"))?;

        if idx > 0 {
            // Ensure a blank line between templates.
            writeln!(output_file)?;
        }

        output_file.write_all(contents.as_bytes())?;

        // Ensure each appended template ends with a newline.
        if !contents.ends_with('\n') {
            writeln!(output_file)?;
        }
    }

    append_general_ignores(&mut output_file)?;

    if selected_languages.len() == 1 {
        println!("✓ Created .gitignore for {}", selected_languages[0]);
    } else {
        println!("✓ Created .gitignore by appending {} templates", selected_languages.len());
    }

    Ok(())
}

fn validate_languages(languages: &[String], language_map: &LanguageMap) -> (Vec<String>, Vec<String>) {
    let mut validated_languages = Vec::new();
    let mut invalid_languages = Vec::new();

    for lang in languages {
        let matched = language_map.keys().find(|k| k.eq_ignore_ascii_case(lang));

        if let Some(matched_key) = matched {
            validated_languages.push(matched_key.clone());
        } else {
            invalid_languages.push(lang.clone());
        }
    }

    (validated_languages, invalid_languages)
}

fn get_gitignore_path() -> Result<PathBuf, Box<dyn StdError>> {
    Ok(current_dir()?.join(".gitignore"))
}

fn create_gitignore_and_print(
    selected_languages: &[String],
    language_map: &LanguageMap,
    output_path: &Path,
) -> Result<(), Box<dyn StdError>> {
    write_gitignore(selected_languages, language_map, output_path)?;
    println!("Languages: {}", selected_languages.join(", "));
    println!("Output: {}", output_path.display());
    Ok(())
}

#[expect(clippy::too_many_lines, reason = "startup flow is kept in one place to avoid over-splitting")]
fn main() {
    let cli = Cli::parse();

    if let Some(languages) = cli.languages {
        let cache_dir = match cache_gitignore_repo() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error downloading archive: {e}");
                return;
            }
        };

        let language_map = match scan_gitignore_templates(&cache_dir) {
            Ok(map) => map,
            Err(e) => {
                eprintln!("Error scanning templates: {e}");
                return;
            }
        };

        let (validated_languages, invalid_languages) = validate_languages(&languages, &language_map);

        if !invalid_languages.is_empty() {
            eprintln!("Error: The following languages were not found:");
            for lang in invalid_languages {
                eprintln!("  - {lang}");
            }
            eprintln!("\nRun without --languages to see available options.");
            return;
        }

        let output_path = match get_gitignore_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error resolving output path: {e}");
                return;
            }
        };

        if let Err(e) = create_gitignore_and_print(&validated_languages, &language_map, &output_path) {
            eprintln!("Error writing .gitignore: {e}");
        }
        return;
    }

    let cache_paths = match cache_paths() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error preparing cache directory: {e}");
            return;
        }
    };

    let needs_background_refresh =
        if has_cached_templates(&cache_paths) { !check_last_synced(&cache_paths.last_synced_path) } else { false };

    let cache_dir = if has_cached_templates(&cache_paths) {
        cache_paths.extract_dir.clone()
    } else {
        match refresh_gitignore_repo(&cache_paths) {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error downloading archive: {e}");
                return;
            }
        }
    };

    let language_map = match scan_gitignore_templates(&cache_dir) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error scanning templates: {e}");
            return;
        }
    };

    let fetch_status =
        Arc::new(Mutex::new(if needs_background_refresh { FetchStatus::Fetching } else { FetchStatus::Idle }));

    if needs_background_refresh {
        let refresh_paths = cache_paths;
        let refresh_status = Arc::clone(&fetch_status);
        thread::spawn(move || {
            if refresh_gitignore_repo(&refresh_paths).is_ok()
                && let Ok(mut status) = refresh_status.lock()
            {
                *status = FetchStatus::Fetched;
            }
        });
    }

    let languages: Vec<String> = language_map.keys().cloned().collect();

    if languages.is_empty() {
        eprintln!("No .gitignore templates found!");
        return;
    }

    let selected = match ui::run_ui(languages, fetch_status) {
        Ok(Some(selected)) => selected,
        Ok(None) => {
            println!("Cancelled.");
            return;
        }
        Err(e) => {
            eprintln!("Error running UI: {e}");
            return;
        }
    };

    if selected.is_empty() {
        println!("No languages selected.");
        return;
    }

    // Get current directory and create .gitignore there
    let output_path = match get_gitignore_path() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error resolving output path: {e}");
            return;
        }
    };

    if let Err(e) = create_gitignore_and_print(&selected, &language_map, &output_path) {
        eprintln!("Error writing .gitignore: {e}");
    }
}
