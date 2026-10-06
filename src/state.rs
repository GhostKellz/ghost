use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// What Ghost deployed: `~/.local/state/ghost/state.json`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    /// `git describe` of the checkout that was deployed, when available.
    pub revision: Option<String>,
    pub components: Vec<String>,
    /// Destination path to the SHA-256 of the content Ghost wrote there.
    pub files: BTreeMap<PathBuf, String>,
    /// Source name to the commit last installed from it.
    #[serde(default)]
    pub sources: BTreeMap<String, String>,
}

pub const STATE_VERSION: u32 = 1;

/// `$XDG_STATE_HOME/ghost`, defaulting to `~/.local/state/ghost`.
pub fn dir(home: &Path, xdg_state_home: Option<std::ffi::OsString>) -> PathBuf {
    xdg_state_home
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .unwrap_or_else(|| home.join(".local/state"))
        .join("ghost")
}

impl State {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match fs::read_to_string(path) {
            Ok(text) => {
                serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self {
                version: STATE_VERSION,
                ..Self::default()
            }),
            Err(err) => Err(err).with_context(|| format!("reading {}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        write_json(path, self)
    }
}

/// Creates `dir` (and missing parents) as 0700 and tightens it if it already
/// exists: state and backups hold copies of the user's config files.
pub fn private_dir(dir: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .with_context(|| format!("creating {}", dir.display()))?;
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
        .with_context(|| format!("restricting {}", dir.display()))
}

/// Writes JSON through a temporary file and rename, so a crash never leaves half a file.
pub fn write_json(path: &Path, value: &impl Serialize) -> anyhow::Result<()> {
    let dir = path.parent().context("state path has no parent")?;
    private_dir(dir)?;
    let tmp = temp_sibling(path);
    fs::write(&tmp, serde_json::to_string_pretty(value)? + "\n")
        .with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

/// `dir/.name.ghost-tmp`, beside `path` so the final rename stays on one filesystem.
pub fn temp_sibling(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.ghost-tmp"))
}

/// SHA-256 of a regular file's content (symlinks are followed); `None` if it doesn't exist.
pub fn hash_file(path: &Path) -> anyhow::Result<Option<String>> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("opening {}", path.display())),
    };
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("reading {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(Some(hex(&hasher.finalize())))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_known_vector() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-fixtures/hash");
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("abc");
        fs::write(&file, "abc").unwrap();
        assert_eq!(
            hash_file(&file).unwrap().unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(hash_file(&dir.join("missing")).unwrap(), None);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn missing_state_is_empty_and_round_trips() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/test-fixtures/state");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("nested/state.json");
        let mut state = State::load(&path).unwrap();
        assert_eq!(state.version, STATE_VERSION);
        assert!(state.files.is_empty());
        state
            .files
            .insert(PathBuf::from("/h/.config/a"), "00".into());
        state.save(&path).unwrap();
        assert_eq!(State::load(&path).unwrap(), state);
        assert!(!temp_sibling(&path).exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
