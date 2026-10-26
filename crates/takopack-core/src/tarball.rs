use std::path::Path;

use flate2::read::GzDecoder;
use tar::Archive;

pub fn extract_tar_gz(archive_path: &Path, extract_dir: &Path) -> Result<(), std::io::Error> {
    let f = std::fs::File::open(archive_path)?;
    let gz = GzDecoder::new(f);
    let mut tar = Archive::new(gz);
    tar.unpack(extract_dir)?;
    Ok(())
}
