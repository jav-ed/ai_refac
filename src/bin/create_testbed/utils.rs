use std::fs;
use std::io::Write;
use std::path::Path;

pub fn create_file(dir: &Path, name: &str, content: &str) -> std::io::Result<()> {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}
