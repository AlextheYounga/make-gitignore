use reqwest;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use zip::ZipArchive;


fn cache_gitignore_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
	let url = "https://github.com/github/gitignore/archive/refs/heads/main.zip";
	let cache_dir = dirs::cache_dir()
		.ok_or("Could not find cache directory")?;
	let app_cache_dir = cache_dir.join("make-gitignore");
	// Ensure cache dir exists
	fs::create_dir_all(&app_cache_dir)?;
	let last_synced_path = app_cache_dir.join("last_synced");
	let zip_path = app_cache_dir.join("gitignore-main.zip");

	// If we have a cached zip and a last_synced timestamp younger than 24h, reuse it
	if last_synced_path.exists() && zip_path.exists() {
		if let Ok(s) = fs::read_to_string(&last_synced_path) {
			if let Ok(ts) = s.trim().parse::<u64>() {
				let last = UNIX_EPOCH + Duration::from_secs(ts);
				if SystemTime::now()
					.duration_since(last)
					.map(|d| d.as_secs() < 24 * 3600)
					.unwrap_or(false)
				{
					println!("Using cached gitignore archive: {:?}", zip_path);
					return Ok(zip_path);
				}
			}
		}
	}

	// Otherwise download and update the timestamp
	let bytes = reqwest::blocking::get(url)?
		.bytes()?;

	// Save to cache directory
	let mut file = File::create(&zip_path)?;
	file.write_all(&bytes)?;

	// Write last_synced timestamp (seconds since UNIX_EPOCH)
	let now_secs = SystemTime::now()
		.duration_since(UNIX_EPOCH)?
		.as_secs();
	fs::write(&last_synced_path, now_secs.to_string())?;
	
	println!("Downloaded gitignore archive to: {:?}", zip_path);
    
    Ok(zip_path)
}

fn unzip_gitignore(zip_path: &Path, cache_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
	let extract_dir = cache_dir.join("gitignore-templates");
	
	// Remove old extraction if it exists
	if extract_dir.exists() {
		fs::remove_dir_all(&extract_dir)?;
	}
	
	let file = File::open(zip_path)?;
	let mut archive = ZipArchive::new(file)?;
	
	println!("Extracting {} files...", archive.len());
	
	for i in 0..archive.len() {
		let mut file = archive.by_index(i)?;
		let outpath = extract_dir.join(file.name());
		
		if file.name().ends_with('/') {
			// It's a directory
			fs::create_dir_all(&outpath)?;
		} else {
			// It's a file
			if let Some(parent) = outpath.parent() {
				fs::create_dir_all(parent)?;
			}
			let mut outfile = File::create(&outpath)?;
			io::copy(&mut file, &mut outfile)?;
		}
		
		// Set permissions on Unix
		#[cfg(unix)]
		{
			use std::os::unix::fs::PermissionsExt;
			if let Some(mode) = file.unix_mode() {
				fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
			}
		}
	}
	
	println!("Extracted to: {:?}", extract_dir);
	Ok(extract_dir)
}

fn main() {
	let zip_path = match cache_gitignore_repo() {
		Ok(path) => path,
		Err(e) => {
			eprintln!("Error downloading archive: {}", e);
			return;
		}
	};
	
	let cache_dir = dirs::cache_dir()
		.expect("Could not find cache directory")
		.join("make-gitignore");
	
	match unzip_gitignore(&zip_path, &cache_dir) {
		Ok(extract_dir) => println!("Done! Templates available at: {:?}", extract_dir),
		Err(e) => eprintln!("Error extracting archive: {}", e),
	}
}

