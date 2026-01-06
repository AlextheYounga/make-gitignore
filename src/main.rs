use reqwest;
use std::fs::{ self, File };
use std::io::Write;
use std::path::{ Path, PathBuf };
use std::time::{ SystemTime, UNIX_EPOCH, Duration };
use zip::ZipArchive;

fn check_last_synced(last_synced_path: &PathBuf) -> bool {
    let timestamp_secs = fs
        ::read_to_string(&last_synced_path)
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    let last_sync_time = timestamp_secs.map(|ts| UNIX_EPOCH + Duration::from_secs(ts));
    let is_cache_fresh = last_sync_time
        .and_then(|last| SystemTime::now().duration_since(last).ok())
        .map(|duration| duration.as_secs() < 24 * 3600)
        .unwrap_or(false);
	return is_cache_fresh;
}

fn cache_gitignore_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let url = "https://github.com/github/gitignore/archive/refs/heads/main.zip";
    let cache_dir = dirs::cache_dir().ok_or("Could not find cache directory")?;
    let app_cache_dir = cache_dir.join("make-gitignore");
    // Ensure cache dir exists
    fs::create_dir_all(&app_cache_dir)?;
	let zip_path = app_cache_dir.join("gitignore-main.zip");

	// If we have a cached zip and a last_synced timestamp younger than 24h, reuse it
    let last_synced_path = app_cache_dir.join("last_synced");
    let cache_exists = last_synced_path.exists() && zip_path.exists();
	let is_cache_fresh = check_last_synced(&last_synced_path);
    if cache_exists && is_cache_fresh {
        println!("Using cached gitignore archive: {:?}", zip_path);
        return Ok(zip_path);
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

	let extract_dir = unzip_gitignore_repo(&zip_path, &cache_dir)?;
    Ok(extract_dir)
}

fn unzip_gitignore_repo(
    zip_path: &Path,
    cache_dir: &Path
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let extract_dir = cache_dir.join("gitignore-templates");

    // Remove old extraction if it exists
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir)?;
    }

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    archive.extract(&extract_dir)?;
    Ok(extract_dir)
}

fn main() {
    let cached = match cache_gitignore_repo() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error downloading archive: {}", e);
            return;
        }
    };

	if cached.exists() {

	}


}
