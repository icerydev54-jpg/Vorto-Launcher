// src/launcher.rs
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Deserialize, Debug)]
struct RemoteVersionInfo {
    version: String,
    download_url: String,
}

// Find the local write-accessible AppData folder (No Admin Prompts Required!)
fn get_vorto_appdata_dir() -> PathBuf {
    let mut path = dirs_next::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("Vorto");
    fs::create_dir_all(&path).unwrap();
    path
}

fn get_local_version(dir: &Path) -> String {
    let version_file = dir.join("local_version.txt");
    if version_file.exists() {
        fs::read_to_string(version_file).unwrap_or_default().trim().to_string()
    } else {
        "0.0.0".to_string()
    }
}

fn set_local_version(dir: &Path, version: &str) -> io::Result<()> {
    fs::write(dir.join("local_version.txt"), version)
}

fn perform_upgrade(vorto_dir: &Path, download_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📥 Downloading new engine files from: {}...", download_url);
    
    // Download zip package
    let response = reqwest::blocking::get(download_url)?;
    let mut content = Vec::new();
    response.error_for_status()?.copy_to(&mut content)?;

    println!("📦 Unpacking archive components...");
    let mut archive = zip::ZipArchive::new(Cursor::new(content))?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => vorto_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    println!("✅ Upgrade completed successfully.");
    Ok(())
}

fn main() {
    println!("🚀 Starting Vorto Launcher...");
    let vorto_dir = get_vorto_appdata_dir();
    let local_ver = get_local_version(&vorto_dir);
    println!("🔍 Current Installed Version: {}", local_ver);

    // This points directly to the version.json in your main repository
    let check_url = "https://raw.githubusercontent.com/icerydev54-jpg/Vorto/main/version.json";
    
    match reqwest::blocking::get(check_url).and_then(|r| r.json::<RemoteVersionInfo>()) {
        Ok(remote_info) => {
            println!("📡 Remote version available: {}", remote_info.version);
            if remote_info.version != local_ver {
                println!("🆕 Update found! Upgrading from {} to {}...", local_ver, remote_info.version);
                match perform_upgrade(&vorto_dir, &remote_info.download_url) {
                    Ok(_) => {
                        let _ = set_local_version(&vorto_dir, &remote_info.version);
                    }
                    Err(e) => {
                        eprintln!("❌ Upgrade failed: {}. Attempting fallback execution...", e);
                    }
                }
            } else {
                println!("✨ Vorto is up to date!");
            }
        }
        Err(e) => {
            eprintln!("⚠️ Offline Mode: Could not reach update server ({}). Using cache...", e);
        }
    }

    // Determine target executable name
    let engine_exe = if cfg!(target_os = "windows") {
        vorto_dir.join("vorto_engine.exe")
    } else {
        vorto_dir.join("vorto_engine")
    };

    if engine_exe.exists() {
        println!("🎮 Running Vorto Studio...");
        
        #[cfg(target_os = "unix")]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&engine_exe).unwrap().permissions();
            perms.set_mode(0o755); // Make sure Mac/Linux has permissions to run it
            let _ = fs::set_permissions(&engine_exe, perms);
        }

        let mut child = Command::new(engine_exe)
            .current_dir(vorto_dir)
            .spawn()
            .expect("Failed to start...");
            
        let _ = child.wait();
    } else {
        eprintln!("❌ Error: Vorto Engine could not be found! Please connect to the internet to download it.");
    }
}