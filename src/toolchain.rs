use std::env::consts::{ARCH, OS};
use std::fs::File;
use std::io::{BufReader, copy};
use std::path::PathBuf;

pub fn install(
    provider: &str,
    user: &str,
    repo: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let current_os = match OS {
        "windows" => "windows",
        "macos" => "darwin",
        "linux" => "linux",
        other => other,
    };

    let current_arch = match ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };

    let extension = if OS == "windows" { "zip" } else { "tar.gz" };

    let url = format!(
        "https://{provider}/{user}/{repo}/releases/latest/download/{repo}-{current_os}-{current_arch}.{extension}"
    );

    println!("Detected Target: OS={}, ARCH={}", OS, ARCH);
    println!("Downloading release from: {}", url);

    let client = reqwest::blocking::Client::new();
    let response = client.get(&url).send()?;

    if !response.status().is_success() {
        return Err(format!("Server returned an error status: {}", response.status()).into());
    }

    let target_dir = tempfile::tempdir()?.keep();
    let target_filename = format!("{repo}-{current_os}-{current_arch}.{extension}");
    let archive_path = target_dir.join(&target_filename);

    let mut archive_file = File::create(&archive_path)?;
    let mut content = response;
    let bytes = copy(&mut content, &mut archive_file)?;
    println!(
        "Successfully downloaded {} bytes to {:?}",
        bytes, archive_path
    );

    let file = File::open(&archive_path)?;
    let reader = BufReader::new(file);

    if extension == "zip" {
        println!("Extracting ZIP archive...");
        let mut archive = zip::ZipArchive::new(reader)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent()
                    && !p.exists()
                {
                    std::fs::create_dir_all(p)?;
                }
                let mut outfile = File::create(&outpath)?;
                copy(&mut file, &mut outfile)?;
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                }
            }
        }
    } else if extension == "tar.gz" {
        println!("Extracting TAR.GZ archive...");
        let tar_gz = flate2::read::GzDecoder::new(reader);
        let mut archive = tar::Archive::new(tar_gz);
        archive.unpack(&target_dir)?;
    }

    if let Err(e) = std::fs::remove_file(&archive_path) {
        eprintln!("Warning: Failed to remove temporary archive file: {}", e);
    }

    println!("Extraction completed. Files stored in: {:?}", target_dir);

    Ok(target_dir)
}
