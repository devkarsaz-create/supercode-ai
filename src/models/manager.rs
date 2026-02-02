use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: PathBuf,
    pub format: String,
    pub size: u64,
}

pub struct ModelManager {
    pub dir: PathBuf,
}

impl ModelManager {
    pub fn new(dir: Option<PathBuf>) -> anyhow::Result<Self> {
        let dir = match dir {
            Some(d) => d,
            None => {
                let mut d = dirs::data_dir().ok_or_else(|| anyhow::anyhow!("no data dir available"))?;
                d.push("super-agent");
                d.push("models");
                d
            }
        };
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    #[cfg(test)]
    fn with_temp() -> anyhow::Result<(Self, tempfile::TempDir)> {
        let td = tempfile::tempdir()?;
        Ok((Self { dir: td.path().to_path_buf() }, td))
    }

    pub fn discover(&self) -> anyhow::Result<Vec<ModelInfo>> {
        let mut out = vec![];
        for entry in fs::read_dir(&self.dir)? {
            let e = entry?;
            let p = e.path();
            if p.is_file() {
                let meta = fs::metadata(&p)?;
                let size = meta.len();
                let format = p
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();
                let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("model").to_string();
                out.push(ModelInfo { name, path: p, format, size });
            }
        }
        Ok(out)
    }

    pub fn import(&self, src: &Path) -> anyhow::Result<ModelInfo> {
        if !src.exists() {
            return Err(anyhow::anyhow!("source model not found"));
        }
        let file_name = src.file_name().ok_or_else(|| anyhow::anyhow!("invalid filename"))?;
        let dest = self.dir.join(file_name);
        // copy with streaming so large files OK
        let mut r = fs::File::open(src)?;
        let mut w = fs::File::create(&dest)?;
        std::io::copy(&mut r, &mut w)?;
        let meta = fs::metadata(&dest)?;
        let size = meta.len();
        let format = dest
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_lowercase();
        let name = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("model").to_string();
        Ok(ModelInfo { name, path: dest, format, size })
    }

    pub fn remove(&self, name: &str) -> anyhow::Result<()> {
        for m in self.discover()? {
            if m.name == name {
                fs::remove_file(m.path)?;
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("model not found"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_and_import() -> anyhow::Result<()> {
        let (mgr, td) = ModelManager::with_temp()?;
        let sample = td.path().join("mymodel.gguf");
        std::fs::write(&sample, b"dummy model contents")?;
        // import from another path
        let (mgr2, td2) = ModelManager::with_temp()?;
        let imported = mgr2.import(&sample)?;
        assert!(imported.name == "mymodel");
        let ms = mgr2.discover()?;
        assert_eq!(ms.len(), 1);
        mgr2.remove("mymodel")?;
        assert_eq!(mgr2.discover()?.len(), 0);
        Ok(())
    }
}
