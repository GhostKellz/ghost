//! Pinned git sources: third-party code Ghost fetches at an exact, reviewed
//! commit (the split-monitor-workspaces Lua plugin, Tela icons).
//!
//! A source either is checked out at `dest`, or is checked out in Ghost's
//! cache and `run` is executed inside it as the user. The fetched commit is
//! verified against the pin; tags and branches are never trusted.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};

use crate::manifest::SourceSpec;
use crate::state::State;

/// A source with `~/` already expanded.
#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub name: String,
    pub url: String,
    pub rev: String,
    pub target: Target,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// Keep a checkout of the pinned commit here.
    Dest(PathBuf),
    /// Run this command inside a cache checkout.
    Run(Vec<String>),
}

impl Source {
    pub fn from_spec(spec: &SourceSpec, home: &Path) -> Self {
        let expand = |s: &str| match s.strip_prefix("~/") {
            Some(rest) => home.join(rest).to_string_lossy().into_owned(),
            None => s.to_owned(),
        };
        let target = match (&spec.dest, &spec.run) {
            (Some(dest), _) => Target::Dest(PathBuf::from(expand(dest))),
            (None, Some(run)) => Target::Run(run.iter().map(|a| expand(a)).collect()),
            (None, None) => unreachable!("validated by Manifest::parse"),
        };
        Self {
            name: spec.name.clone(),
            url: spec.url.clone(),
            rev: spec.rev.clone(),
            target,
        }
    }

    pub fn short_rev(&self) -> &str {
        &self.rev[..self.rev.len().min(12)]
    }
}

/// Sources whose pinned commit isn't in place yet.
pub fn pending<'s>(sources: &'s [Source], state: &State) -> Vec<&'s Source> {
    sources
        .iter()
        .filter(|s| {
            let recorded = state.sources.get(&s.name) == Some(&s.rev);
            match &s.target {
                // A checkout can be changed behind Ghost's back; check it.
                Target::Dest(dest) => !recorded || head(dest).as_deref() != Some(s.rev.as_str()),
                Target::Run(_) => !recorded,
            }
        })
        .collect()
}

/// Fetches the pinned commit and installs it; `cache` holds checkouts for `run` sources.
pub fn apply(source: &Source, cache: &Path) -> anyhow::Result<()> {
    match &source.target {
        Target::Dest(dest) => checkout(dest, &source.url, &source.rev),
        Target::Run(command) => {
            let dir = cache.join(&source.name);
            checkout(&dir, &source.url, &source.rev)?;
            let (program, args) = command.split_first().context("empty run command")?;
            let status = Command::new(program)
                .args(args)
                .current_dir(&dir)
                .status()
                .with_context(|| format!("running {program} for {}", source.name))?;
            if !status.success() {
                bail!(
                    "{} for {} failed ({status})",
                    command.join(" "),
                    source.name
                );
            }
            Ok(())
        }
    }
}

/// Makes `dir` a checkout of exactly `rev` from `url`.
fn checkout(dir: &Path, url: &str, rev: &str) -> anyhow::Result<()> {
    if !dir.join(".git").exists() {
        let occupied = fs::read_dir(dir).is_ok_and(|mut d| d.next().is_some());
        if occupied {
            bail!(
                "{} exists and is not a git checkout; move it aside and re-run",
                dir.display()
            );
        }
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        git(dir, &["init", "-q"])?;
    }
    git(dir, &["fetch", "-q", "--depth", "1", url, rev])?;
    git(dir, &["checkout", "-q", "--detach", "FETCH_HEAD"])?;
    match head(dir) {
        Some(got) if got == rev => Ok(()),
        got => bail!(
            "{} is at {} after fetching, expected {rev}",
            dir.display(),
            got.as_deref().unwrap_or("nothing")
        ),
    }
}

fn head(dir: &Path) -> Option<String> {
    if !dir.join(".git").exists() {
        return None;
    }
    git(dir, &["rev-parse", "HEAD"]).ok()
}

fn git(dir: &Path, args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        // Never prompt for credentials: sources are public https repositories.
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(args)
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A local upstream repository with two commits, plus scratch space.
    struct Fixture {
        root: PathBuf,
        upstream: PathBuf,
        revs: Vec<String>,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/test-fixtures/sources")
                .join(name);
            let _ = fs::remove_dir_all(&root);
            let upstream = root.join("upstream");
            fs::create_dir_all(&upstream).unwrap();
            let run = |args: &[&str]| git(&upstream, args).unwrap();
            run(&["init", "-q"]);
            run(&["config", "user.email", "t@example.invalid"]);
            run(&["config", "user.name", "t"]);
            run(&["config", "commit.gpgsign", "false"]);
            // Allow fetching a specific commit, as GitHub does.
            run(&["config", "uploadpack.allowAnySHA1InWant", "true"]);
            let mut revs = Vec::new();
            for (i, text) in ["one", "two"].iter().enumerate() {
                fs::write(upstream.join("file.txt"), text).unwrap();
                fs::write(
                    upstream.join("install.sh"),
                    format!("#!/bin/sh\necho {text} > \"$1/installed.txt\"\n"),
                )
                .unwrap();
                run(&["add", "."]);
                run(&["commit", "-q", "-m", &format!("c{i}")]);
                revs.push(run(&["rev-parse", "HEAD"]));
            }
            Self {
                root,
                upstream,
                revs,
            }
        }

        fn url(&self) -> String {
            format!("file://{}", self.upstream.display())
        }

        fn dest_source(&self, rev: usize) -> Source {
            Source {
                name: "plugin".into(),
                url: self.url(),
                rev: self.revs[rev].clone(),
                target: Target::Dest(self.root.join("home/plugin")),
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn checkout_pin_then_update_in_place() {
        let fx = Fixture::new("dest");
        let cache = fx.root.join("cache");
        let dest = fx.root.join("home/plugin");

        let first = fx.dest_source(0);
        let mut state = State::default();
        assert_eq!(pending(std::slice::from_ref(&first), &state).len(), 1);
        apply(&first, &cache).unwrap();
        assert_eq!(fs::read_to_string(dest.join("file.txt")).unwrap(), "one");
        state.sources.insert(first.name.clone(), first.rev.clone());
        assert!(pending(std::slice::from_ref(&first), &state).is_empty());

        let second = fx.dest_source(1);
        assert_eq!(pending(std::slice::from_ref(&second), &state).len(), 1);
        apply(&second, &cache).unwrap();
        assert_eq!(fs::read_to_string(dest.join("file.txt")).unwrap(), "two");
    }

    #[test]
    fn changed_checkout_is_pending_again() {
        let fx = Fixture::new("drift");
        let source = fx.dest_source(1);
        apply(&source, &fx.root.join("cache")).unwrap();
        let mut state = State::default();
        state
            .sources
            .insert(source.name.clone(), source.rev.clone());
        assert!(pending(std::slice::from_ref(&source), &state).is_empty());
        // Someone checks out a different commit by hand.
        apply(&fx.dest_source(0), &fx.root.join("cache")).unwrap();
        assert_eq!(
            pending(std::slice::from_ref(&source), &state).len(),
            1,
            "checkout moved off the recorded pin"
        );
    }

    #[test]
    fn unknown_rev_fails() {
        let fx = Fixture::new("badrev");
        let mut source = fx.dest_source(0);
        source.rev = "0".repeat(40);
        assert!(apply(&source, &fx.root.join("cache")).is_err());
    }

    #[test]
    fn non_git_destination_is_never_overwritten() {
        let fx = Fixture::new("occupied");
        let dest = fx.root.join("home/plugin");
        fs::create_dir_all(&dest).unwrap();
        fs::write(dest.join("mine.txt"), "keep").unwrap();
        let err = apply(&fx.dest_source(0), &fx.root.join("cache")).unwrap_err();
        assert!(err.to_string().contains("not a git checkout"));
        assert_eq!(fs::read_to_string(dest.join("mine.txt")).unwrap(), "keep");
    }

    #[test]
    fn run_executes_inside_cache_checkout() {
        let fx = Fixture::new("run");
        let out = fx.root.join("out");
        fs::create_dir_all(&out).unwrap();
        let source = Source {
            name: "icons".into(),
            url: fx.url(),
            rev: fx.revs[1].clone(),
            target: Target::Run(vec![
                "sh".into(),
                "./install.sh".into(),
                out.to_string_lossy().into_owned(),
            ]),
        };
        apply(&source, &fx.root.join("cache")).unwrap();
        assert_eq!(
            fs::read_to_string(out.join("installed.txt")).unwrap(),
            "two\n"
        );
        assert!(fx.root.join("cache/icons/.git").exists());
    }
}
