use clap::Parser;
use reqwest;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

mod ui;

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

fn append_general_ignores(output_file: &mut File) -> std::io::Result<()> {
    writeln!(output_file)?;
    writeln!(output_file, "# General")?;

    for line in GENERAL_IGNORE_LINES {
        writeln!(output_file, "{}", line)?;
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
    let timestamp_secs = fs::read_to_string(last_synced_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    let last_sync_time = timestamp_secs.map(|ts| UNIX_EPOCH + Duration::from_secs(ts));
    last_sync_time
        .and_then(|last| SystemTime::now().duration_since(last).ok())
        .map(|duration| duration.as_secs() < 24 * 3600)
        .unwrap_or(false)
}

fn cache_gitignore_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let url = "https://github.com/github/gitignore/archive/refs/heads/main.zip";
    let cache_dir = dirs::cache_dir().ok_or("Could not find cache directory")?;
    let app_cache_dir = cache_dir.join("make-gitignore");
    let extract_dir = cache_dir.join("gitignore-templates");
    // Ensure cache dir exists
    fs::create_dir_all(&app_cache_dir)?;
    let zip_path = app_cache_dir.join("gitignore-main.zip");

    // If we have a cached zip and a last_synced timestamp younger than 24h, reuse it
    let last_synced_path = app_cache_dir.join("last_synced");

    let cache_exists = last_synced_path.exists() && zip_path.exists();
    let is_cache_fresh = check_last_synced(&last_synced_path);
    if cache_exists && is_cache_fresh {
        println!("Using cached gitignore archive: {:?}", extract_dir);
        return Ok(extract_dir);
    }

    // Otherwise download and update the timestamp
    let bytes = reqwest::blocking::get(url)?.bytes()?;

    // Save to cache directory
    let mut file = File::create(&zip_path)?;
    file.write_all(&bytes)?;

    // Write last_synced timestamp (seconds since UNIX_EPOCH)
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    fs::write(&last_synced_path, now_secs.to_string())?;

    println!("Downloaded gitignore archive to: {:?}", zip_path);

    let extract_dir = unzip_gitignore_repo(&zip_path, extract_dir)?;
    Ok(extract_dir)
}

fn unzip_gitignore_repo(
    zip_path: &Path,
    extract_dir: PathBuf,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Remove old extraction if it exists
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir)?;
    }

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    archive.extract(&extract_dir)?;
    Ok(extract_dir)
}

fn scan_gitignore_templates(
    extract_dir: &Path,
) -> Result<HashMap<String, PathBuf>, Box<dyn std::error::Error>> {
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
        let language = filename.strip_suffix(".gitignore").unwrap().to_string();
        language_map.insert(language, path);
    }

    println!("Found {} gitignore templates", language_map.len());
    Ok(language_map)
}

fn write_gitignore(
    selected_languages: &[String],
    language_map: &HashMap<String, PathBuf>,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if selected_languages.is_empty() {
        return Err("No languages selected".into());
    }

    // Always create .gitignore by appending templates in order, preserving formatting.
    let mut output_file = File::create(output_path)?;

    for (idx, language) in selected_languages.iter().enumerate() {
        let source_path = language_map
            .get(language)
            .ok_or(format!("Language '{}' not found", language))?;

        if idx > 0 {
            // Ensure a blank line between templates.
            writeln!(output_file)?;
        }

        let contents = fs::read_to_string(source_path)?;
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
        println!(
            "✓ Created .gitignore by appending {} templates",
            selected_languages.len()
        );
    }

    Ok(())
}

fn validate_languages(
    languages: &[String],
    language_map: &HashMap<String, PathBuf>,
) -> (Vec<String>, Vec<String>) {
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

fn get_gitignore_path() -> PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join(".gitignore")
}

fn create_gitignore_and_print(
    selected_languages: &[String],
    language_map: &HashMap<String, PathBuf>,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    write_gitignore(selected_languages, language_map, output_path)?;
    println!("Languages: {}", selected_languages.join(", "));
    println!("Output: {:?}", output_path);
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    let cache_dir = match cache_gitignore_repo() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error downloading archive: {}", e);
            return;
        }
    };

    let language_map = match scan_gitignore_templates(&cache_dir) {
        Ok(map) => map,
        Err(e) => {
            eprintln!("Error scanning templates: {}", e);
            return;
        }
    };

    // Handle CLI arguments if provided - exit early
    if let Some(languages) = cli.languages {
        let (validated_languages, invalid_languages) =
            validate_languages(&languages, &language_map);

        if !invalid_languages.is_empty() {
            eprintln!("Error: The following languages were not found:");
            for lang in invalid_languages {
                eprintln!("  - {}", lang);
            }
            eprintln!("\nRun without --languages to see available options.");
            return;
        }

        // Proceed with validated languages
        let output_path = get_gitignore_path();

        if let Err(e) =
            create_gitignore_and_print(&validated_languages, &language_map, &output_path)
        {
            eprintln!("Error writing .gitignore: {}", e);
        }
        return;
    }

    // No arguments provided - run the UI
    let languages: Vec<String> = language_map.keys().cloned().collect();

    if languages.is_empty() {
        eprintln!("No .gitignore templates found!");
        return;
    }

    let selected = match ui::run_ui(languages) {
        Ok(Some(selected)) => selected,
        Ok(None) => {
            println!("Cancelled.");
            return;
        }
        Err(e) => {
            eprintln!("Error running UI: {}", e);
            return;
        }
    };

    if selected.is_empty() {
        println!("No languages selected.");
        return;
    }

    // Get current directory and create .gitignore there
    let output_path = get_gitignore_path();

    if let Err(e) = create_gitignore_and_print(&selected, &language_map, &output_path) {
        eprintln!("Error writing .gitignore: {}", e);
    }
}
