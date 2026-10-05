use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

/// The repository's `ghost.toml`, embedded so `ghost doctor` works without a checkout.
pub const EMBEDDED: &str = include_str!("../ghost.toml");

/// `ghost.toml`: the components Ghost can deploy.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub components: BTreeMap<String, Component>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub description: String,
    #[serde(default)]
    pub required: bool,
    /// Installed unless excluded; `false` makes the component opt-in.
    #[serde(default = "yes")]
    pub default: bool,
    /// Files go to system paths and are written with sudo.
    #[serde(default)]
    pub root: bool,
    #[serde(default)]
    pub packages: Vec<String>,
    #[serde(default)]
    pub files: Vec<Mapping>,
    /// Commands run with sudo after this component's files change.
    #[serde(default)]
    pub hooks: Vec<Vec<String>>,
}

fn yes() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub src: PathBuf,
    pub dest: String,
}

/// One file to deploy: where it comes from and where it goes.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub src: PathBuf,
    pub dest: PathBuf,
}

impl Manifest {
    pub fn load(source: &Path) -> anyhow::Result<Self> {
        let path = source.join("ghost.toml");
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("parsing {}", path.display()))
    }

    pub fn parse(text: &str) -> anyhow::Result<Self> {
        let manifest: Self = toml::from_str(text)?;
        for (name, component) in &manifest.components {
            for mapping in &component.files {
                if mapping.src.is_absolute()
                    || mapping
                        .src
                        .components()
                        .any(|c| c == std::path::Component::ParentDir)
                {
                    bail!(
                        "component {name}: src {} must stay inside the repository",
                        mapping.src.display()
                    );
                }
                // User components may only touch the home directory, and root
                // components only system paths, so sudo is never used on ~/.
                let ok = if component.root {
                    mapping.dest.starts_with('/')
                } else {
                    mapping.dest.starts_with("~/")
                };
                if !ok || mapping.dest.contains("/../") {
                    bail!(
                        "component {name}: dest {} must {}",
                        mapping.dest,
                        if component.root {
                            "be an absolute system path"
                        } else {
                            "start with ~/"
                        }
                    );
                }
            }
            if component.hooks.iter().any(Vec::is_empty) {
                bail!("component {name}: empty hook command");
            }
            if !component.hooks.is_empty() && !component.root {
                bail!("component {name}: hooks run with sudo, so the component must be root");
            }
        }
        Ok(manifest)
    }

    /// Required and default components, plus `with`, minus `without`.
    pub fn select(&self, with: &[String], without: &[String]) -> anyhow::Result<Vec<String>> {
        for name in with.iter().chain(without) {
            if !self.components.contains_key(name) {
                let known: Vec<&str> = self.components.keys().map(String::as_str).collect();
                bail!("unknown component {name:?} (known: {})", known.join(", "));
            }
        }
        for name in without {
            if self.components[name].required {
                bail!("component {name:?} is required");
            }
            if with.contains(name) {
                bail!("component {name:?} is both included and excluded");
            }
        }
        Ok(self
            .components
            .iter()
            .filter(|(name, c)| {
                c.required || with.contains(name) || (c.default && !without.contains(name))
            })
            .map(|(name, _)| name.clone())
            .collect())
    }

    /// Every destination of every root component: the only system paths Ghost
    /// may change with sudo, whether or not the component is selected (so a
    /// deselected component's files can still be removed or restored).
    pub fn root_destinations(&self) -> Vec<PathBuf> {
        self.components
            .values()
            .filter(|c| c.root)
            .flat_map(|c| c.files.iter().map(|m| PathBuf::from(&m.dest)))
            .collect()
    }

    /// Packages of the chosen components, deduplicated, in manifest order.
    pub fn packages(&self, chosen: &[String]) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for name in chosen {
            for package in &self.components[name].packages {
                if !out.contains(&package.as_str()) {
                    out.push(package);
                }
            }
        }
        out
    }

    /// Every file of the chosen components, directories expanded, sorted by destination.
    pub fn entries(
        &self,
        source: &Path,
        home: &Path,
        chosen: &[String],
    ) -> anyhow::Result<Vec<Entry>> {
        let mut entries = Vec::new();
        for name in chosen {
            for mapping in &self.components[name].files {
                let src = source.join(&mapping.src);
                let dest = match mapping.dest.strip_prefix("~/") {
                    Some(rest) => home.join(rest),
                    None => PathBuf::from(&mapping.dest),
                };
                collect(&src, &dest, &mut entries)
                    .with_context(|| format!("component {name}: {}", src.display()))?;
            }
        }
        entries.sort_by(|a, b| a.dest.cmp(&b.dest));
        if let Some(pair) = entries.windows(2).find(|pair| pair[0].dest == pair[1].dest) {
            bail!("two sources map to {}", pair[0].dest.display());
        }
        Ok(entries)
    }
}

fn collect(src: &Path, dest: &Path, out: &mut Vec<Entry>) -> anyhow::Result<()> {
    let meta = fs::symlink_metadata(src)?;
    if meta.is_dir() {
        for child in fs::read_dir(src)? {
            let child = child?;
            collect(&child.path(), &dest.join(child.file_name()), out)?;
        }
    } else if meta.is_file() {
        out.push(Entry {
            src: src.to_owned(),
            dest: dest.to_owned(),
        });
    } else {
        bail!("{} is not a regular file or directory", src.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_manifest_parses_and_every_source_exists() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest = Manifest::load(source).unwrap();
        let all: Vec<String> = manifest.components.keys().cloned().collect();
        let entries = manifest
            .entries(source, Path::new("/home/test"), &all)
            .unwrap();
        assert!(
            entries
                .iter()
                .any(|e| e.dest == Path::new("/home/test/.config/hypr/hyprland.lua"))
        );
        assert!(
            entries
                .iter()
                .any(|e| e.dest == Path::new("/usr/share/sddm/themes/ghost/Main.qml"))
        );
        assert!(entries.iter().all(|e| e.src.is_file()));
        assert_eq!(
            Manifest::parse(EMBEDDED).unwrap().components.len(),
            manifest.components.len()
        );
    }

    #[test]
    fn default_selection_and_packages() {
        let manifest = Manifest::parse(EMBEDDED).unwrap();
        let chosen = manifest.select(&[], &[]).unwrap();
        assert_eq!(chosen, ["apps", "core", "shell"]);
        let packages = manifest.packages(&chosen);
        assert!(packages.contains(&"hyprland") && packages.contains(&"waybar"));
        assert!(!packages.contains(&"sddm"));
        let mut sorted = packages.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), packages.len(), "no duplicates");
    }

    const SAMPLE: &str = r#"
        [components.core]
        description = "core"
        required = true
        packages = ["a", "b"]
        [components.extra]
        description = "extra"
        packages = ["b", "c"]
        [components.optin]
        description = "opt-in"
        default = false
    "#;

    #[test]
    fn selection_rules() {
        let manifest = Manifest::parse(SAMPLE).unwrap();
        assert_eq!(manifest.select(&[], &[]).unwrap(), ["core", "extra"]);
        assert_eq!(
            manifest.select(&["optin".into()], &[]).unwrap(),
            ["core", "extra", "optin"]
        );
        assert_eq!(manifest.select(&[], &["extra".into()]).unwrap(), ["core"]);
        assert!(manifest.select(&[], &["core".into()]).is_err());
        assert!(manifest.select(&["nope".into()], &[]).is_err());
        assert!(
            manifest
                .select(&["extra".into()], &["extra".into()])
                .is_err()
        );
        assert_eq!(
            manifest.packages(&["core".into(), "extra".into()]),
            ["a", "b", "c"]
        );
    }

    #[test]
    fn rejects_unsafe_mappings() {
        let component = |extra: &str, src: &str, dest: &str| {
            format!(
                "[components.a]\ndescription = \"a\"\n{extra}\n[[components.a.files]]\nsrc = \"{src}\"\ndest = \"{dest}\"\n"
            )
        };
        assert!(Manifest::parse(&component("", "../etc", "~/x")).is_err());
        assert!(Manifest::parse(&component("", "x", "x")).is_err());
        assert!(
            Manifest::parse(&component("", "x", "/etc/x")).is_err(),
            "user component writing a system path"
        );
        assert!(
            Manifest::parse(&component("root = true", "x", "~/x")).is_err(),
            "root component writing into home"
        );
        assert!(Manifest::parse(&component("root = true", "x", "/etc/../root/x")).is_err());
        assert!(Manifest::parse(&component("root = true", "x", "/etc/x")).is_ok());
        assert!(Manifest::parse("[components.a]\ndescription = \"a\"\nrequried = true\n").is_err());
        assert!(
            Manifest::parse("[components.a]\ndescription = \"a\"\nhooks = [[\"true\"]]\n").is_err(),
            "hooks need root"
        );
    }
}
