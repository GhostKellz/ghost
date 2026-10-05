//! File deployment: three-way planning against recorded state, backups, and restore.
//!
//! For each managed file Ghost compares three hashes: the new source, the
//! file on disk, and what Ghost last wrote there (from state). Untouched
//! files are updated, files you edited are left alone with the new version
//! written beside them as `<name>.ghost-new`, and anything replaced or
//! removed is copied into a backup first.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};

use crate::manifest::Entry;
use crate::state::{State, hash_file, private_dir, temp_sibling, write_json};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    /// Not on disk yet.
    Create,
    /// Ghost's previous version, unedited: replaced.
    Update,
    /// Exists but Ghost never deployed it: backed up, then replaced.
    Replace,
    /// You edited it and Ghost's version changed: new version written beside it.
    Conflict,
    /// Ghost no longer ships it and you never edited it: backed up, then removed.
    Remove,
    /// Already identical.
    Keep,
    /// A conflict whose `.ghost-new` already holds this version, awaiting your review.
    Pending,
    /// You edited it; Ghost's version is unchanged. Left alone.
    KeepEdited,
    /// You deleted a file Ghost deployed. Not recreated.
    SkipDeleted,
    /// Ghost no longer ships it but you edited it. Left in place, no longer tracked.
    Release,
    /// Ghost no longer ships it and it's already gone. Dropped from state.
    Forget,
}

impl Kind {
    pub fn changes_disk(self) -> bool {
        matches!(
            self,
            Kind::Create | Kind::Update | Kind::Replace | Kind::Conflict | Kind::Remove
        )
    }

    fn label(self) -> &'static str {
        match self {
            Kind::Create => "create",
            Kind::Update => "update",
            Kind::Replace => "replace (backed up)",
            Kind::Conflict => "edited by you; new version saved as .ghost-new",
            Kind::Remove => "remove (backed up)",
            Kind::Keep => "unchanged",
            Kind::Pending => "edited by you; .ghost-new still waiting for review",
            Kind::KeepEdited => "edited by you; kept",
            Kind::SkipDeleted => "deleted by you; not recreated",
            Kind::Release => "no longer shipped; kept your edited copy",
            Kind::Forget => "no longer shipped; already gone",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Action {
    pub kind: Kind,
    pub dest: PathBuf,
    pub src: Option<PathBuf>,
    /// Hash of the source content to be deployed.
    pub hash: Option<String>,
}

#[derive(Debug, Default)]
pub struct Plan {
    pub actions: Vec<Action>,
}

impl Plan {
    pub fn has_changes(&self) -> bool {
        self.actions.iter().any(|a| a.kind.changes_disk())
    }

    pub fn count(&self, kind: Kind) -> usize {
        self.actions.iter().filter(|a| a.kind == kind).count()
    }

    /// Changes listed individually, unchanged files summarised; paths shown relative to `home`.
    pub fn render(&self, home: &Path) -> String {
        let mut out = String::new();
        let mut kinds: Vec<Kind> = self.actions.iter().map(|a| a.kind).collect();
        kinds.sort();
        kinds.dedup();
        for kind in kinds {
            if kind == Kind::Keep {
                continue;
            }
            let _ = writeln!(out, "{}:", kind.label());
            for action in self.actions.iter().filter(|a| a.kind == kind) {
                let _ = writeln!(out, "  {}", display(&action.dest, home));
            }
        }
        let unchanged = self.count(Kind::Keep);
        if unchanged > 0 {
            let _ = writeln!(out, "{unchanged} file(s) already up to date");
        }
        out
    }
}

pub fn display(path: &Path, home: &Path) -> String {
    match path.strip_prefix(home) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

pub fn ghost_new(dest: &Path) -> PathBuf {
    let mut name = dest.file_name().unwrap_or_default().to_os_string();
    name.push(".ghost-new");
    dest.with_file_name(name)
}

pub fn plan(entries: &[Entry], state: &State) -> anyhow::Result<Plan> {
    let mut actions = Vec::new();
    for entry in entries {
        let new =
            hash_file(&entry.src)?.with_context(|| format!("{} vanished", entry.src.display()))?;
        let deployed = state.files.get(&entry.dest);
        let kind = match fs::symlink_metadata(&entry.dest) {
            Err(_) => {
                if deployed.is_some() {
                    Kind::SkipDeleted
                } else {
                    Kind::Create
                }
            }
            Ok(meta) if meta.is_dir() => {
                bail!("{} is a directory; expected a file", entry.dest.display())
            }
            Ok(meta) => {
                let current = hash_file(&entry.dest)?.unwrap_or_default();
                match deployed {
                    _ if current == new => Kind::Keep,
                    // Never write through or replace a symlink you set up (e.g. a dotfiles repo).
                    _ if meta.file_type().is_symlink() => Kind::Conflict,
                    None => Kind::Replace,
                    Some(d) if *d == current => Kind::Update,
                    Some(d) if *d == new => Kind::KeepEdited,
                    Some(_) => Kind::Conflict,
                }
            }
        };
        let kind = if kind == Kind::Conflict
            && hash_file(&ghost_new(&entry.dest))?.as_ref() == Some(&new)
        {
            Kind::Pending
        } else {
            kind
        };
        actions.push(Action {
            kind,
            dest: entry.dest.clone(),
            src: Some(entry.src.clone()),
            hash: Some(new),
        });
    }

    for (dest, deployed) in &state.files {
        if entries.iter().any(|e| &e.dest == dest) {
            continue;
        }
        let kind = match hash_file(dest)? {
            None => Kind::Forget,
            Some(current) if current == *deployed => Kind::Remove,
            Some(_) => Kind::Release,
        };
        actions.push(Action {
            kind,
            dest: dest.clone(),
            src: None,
            hash: None,
        });
    }
    Ok(Plan { actions })
}

/// Record of one deployment, enough to undo it.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub revision: Option<String>,
    pub state_before: State,
    /// Files Ghost created, with the hash it wrote (removed again on restore if unchanged).
    pub created: Vec<(PathBuf, String)>,
    /// Files that existed and were replaced or removed; originals under `files/`.
    pub saved: Vec<PathBuf>,
    /// Directories Ghost created, outermost first.
    pub created_dirs: Vec<PathBuf>,
}

impl Backup {
    fn stored(dir: &Path, original: &Path) -> PathBuf {
        dir.join("files")
            .join(original.strip_prefix("/").unwrap_or(original))
    }
}

pub struct Paths {
    pub state: PathBuf,
    pub backups: PathBuf,
}

impl Paths {
    pub fn under(state_dir: &Path) -> Self {
        Self {
            state: state_dir.join("state.json"),
            backups: state_dir.join("backups"),
        }
    }
}

/// Filesystem writes. Paths outside the user's home are written with sudo
/// when enabled; everything else is written directly. Writes go through a
/// temporary sibling and a rename either way.
///
/// Paths reach these methods from user-writable files (state.json, backup
/// indexes), so every sudo operation is confined to the destinations that
/// root components declare in ghost.toml.
pub struct Ops {
    home: PathBuf,
    sudo: bool,
    root_dests: Vec<PathBuf>,
}

/// What a sudo operation is allowed to touch.
#[derive(Clone, Copy, PartialEq)]
enum Reach {
    /// Inside a declared root destination (files and their directories).
    Inside,
    /// Inside, or an ancestor directory that has to exist for one.
    InsideOrAncestor,
}

impl Ops {
    pub fn new(home: &Path, sudo: bool, root_dests: Vec<PathBuf>) -> Self {
        Self {
            home: home.to_owned(),
            sudo,
            root_dests,
        }
    }

    pub fn needs_root(&self, path: &Path) -> bool {
        self.sudo && !path.starts_with(&self.home)
    }

    /// Refuses a sudo operation outside the declared root destinations. `..`
    /// is rejected outright: `Path::starts_with` compares components
    /// literally, so `dest/../../etc` would otherwise pass.
    fn check_root(&self, path: &Path, reach: Reach) -> anyhow::Result<()> {
        let escapes = path
            .components()
            .any(|c| c == std::path::Component::ParentDir);
        let inside = self.root_dests.iter().any(|d| path.starts_with(d));
        let ancestor =
            reach == Reach::InsideOrAncestor && self.root_dests.iter().any(|d| d.starts_with(path));
        if !path.is_absolute() || escapes || !(inside || ancestor) {
            bail!(
                "refusing to change {} with sudo: not inside a root component's destination in ghost.toml",
                path.display()
            );
        }
        Ok(())
    }

    /// Copies `src` to `dest` keeping its permissions (e.g. the executable bit on scripts).
    fn install(&self, src: &Path, dest: &Path) -> anyhow::Result<()> {
        let tmp = temp_sibling(dest);
        if self.needs_root(dest) {
            self.check_root(dest, Reach::Inside)?;
            let mode = install_mode(src)?;
            sudo(&[
                "install".as_ref(),
                "-m".as_ref(),
                format!("{mode:o}").as_ref(),
                "--".as_ref(),
                src.as_os_str(),
                tmp.as_os_str(),
            ])?;
            return sudo(&[
                "mv".as_ref(),
                "-f".as_ref(),
                "--".as_ref(),
                tmp.as_os_str(),
                dest.as_os_str(),
            ]);
        }
        fs::copy(src, &tmp)
            .with_context(|| format!("copying {} to {}", src.display(), tmp.display()))?;
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(install_mode(src)?))
                .with_context(|| format!("setting permissions on {}", tmp.display()))?;
        }
        fs::rename(&tmp, dest).with_context(|| format!("replacing {}", dest.display()))?;
        Ok(())
    }

    fn remove(&self, path: &Path) -> anyhow::Result<()> {
        if self.needs_root(path) {
            self.check_root(path, Reach::Inside)?;
            return sudo(&[
                "rm".as_ref(),
                "-f".as_ref(),
                "--".as_ref(),
                path.as_os_str(),
            ]);
        }
        fs::remove_file(path).with_context(|| format!("removing {}", path.display()))
    }

    fn mkdir(&self, dir: &Path) -> anyhow::Result<()> {
        if self.needs_root(dir) {
            self.check_root(dir, Reach::InsideOrAncestor)?;
            return sudo(&["mkdir".as_ref(), "--".as_ref(), dir.as_os_str()]);
        }
        fs::create_dir(dir).with_context(|| format!("creating {}", dir.display()))
    }

    /// Removes a directory only if it is empty; anything else is left alone.
    fn rmdir_if_empty(&self, dir: &Path) {
        let empty = fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_none());
        if !empty {
            return;
        }
        if self.needs_root(dir) {
            // Only directories inside a destination: never an ancestor such
            // as /usr/share/sddm/themes, even when empty.
            if self.check_root(dir, Reach::Inside).is_ok() {
                let _ = sudo(&["rmdir".as_ref(), "--".as_ref(), dir.as_os_str()]);
            }
        } else {
            let _ = fs::remove_dir(dir);
        }
    }

    /// Creates missing parent directories and returns the ones it made, outermost first.
    fn create_parents(&self, dest: &Path) -> anyhow::Result<Vec<PathBuf>> {
        let mut missing = Vec::new();
        let mut dir = dest.parent();
        while let Some(d) = dir {
            if d.exists() {
                break;
            }
            missing.push(d.to_owned());
            dir = d.parent();
        }
        missing.reverse();
        for d in &missing {
            self.mkdir(d)?;
        }
        Ok(missing)
    }
}

/// The source's permission bits without group/other write (and without
/// setuid/setgid/sticky): keeps the executable bit on scripts, but never lets
/// a loose checkout produce a world-writable udev rule or config file.
fn install_mode(src: &Path) -> anyhow::Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    let mode = fs::metadata(src)
        .with_context(|| format!("reading {}", src.display()))?
        .permissions()
        .mode();
    Ok(mode & 0o755)
}

fn sudo(args: &[&std::ffi::OsStr]) -> anyhow::Result<()> {
    let status = std::process::Command::new("sudo")
        .args(args)
        .status()
        .context("running sudo")?;
    if !status.success() {
        let shown: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        bail!("sudo {} failed ({status})", shown.join(" "));
    }
    Ok(())
}

/// Applies the plan, backing up first. Progress is saved after every file so
/// an interrupted run can still be restored. Returns the backup id, if any.
pub fn execute(
    plan: &Plan,
    state: &mut State,
    paths: &Paths,
    ops: &Ops,
    revision: Option<String>,
    components: Vec<String>,
) -> anyhow::Result<Option<String>> {
    // Before any original is copied in, and also tightening folders created
    // by older versions: backups must never be readable by other users.
    private_state_dirs(paths)?;
    let mut backup = None;
    if plan.has_changes() {
        let id = new_backup_id(&paths.backups)?;
        backup = Some((
            paths.backups.join(&id),
            Backup {
                id,
                revision: revision.clone(),
                state_before: state.clone(),
                ..Backup::default()
            },
        ));
    }
    // Only after the backup has captured the previous state.
    state.components = components;

    for action in &plan.actions {
        let result = apply(action, state, ops, backup.as_mut());
        if let Some((dir, record)) = &backup {
            write_json(&dir.join("index.json"), record)?;
        }
        state.save(&paths.state)?;
        result.with_context(|| format!("{} {}", action.kind.label(), action.dest.display()))?;
    }
    state.revision = revision;
    state.save(&paths.state)?;
    Ok(backup.map(|(_, record)| record.id))
}

fn apply(
    action: &Action,
    state: &mut State,
    ops: &Ops,
    backup: Option<&mut (PathBuf, Backup)>,
) -> anyhow::Result<()> {
    let dest = &action.dest;
    match action.kind {
        Kind::Create | Kind::Update | Kind::Replace | Kind::Conflict => {
            let (dir, record) = backup.context("change without a backup record")?;
            let src = action.src.as_ref().context("missing source")?;
            let hash = action.hash.clone().context("missing hash")?;
            match action.kind {
                Kind::Create => {
                    record.created_dirs.extend(ops.create_parents(dest)?);
                    ops.install(src, dest)?;
                    record.created.push((dest.clone(), hash.clone()));
                    state.files.insert(dest.clone(), hash);
                }
                Kind::Update | Kind::Replace => {
                    save_original(dir, record, dest)?;
                    ops.install(src, dest)?;
                    state.files.insert(dest.clone(), hash);
                }
                _ => {
                    // Your file stays; state keeps the old hash so the next run
                    // still recognises your edit.
                    let side = ghost_new(dest);
                    if side.exists() {
                        save_original(dir, record, &side)?;
                    } else {
                        record.created.push((side.clone(), hash));
                    }
                    ops.install(src, &side)?;
                }
            }
        }
        Kind::Remove => {
            let (dir, record) = backup.context("change without a backup record")?;
            save_original(dir, record, dest)?;
            ops.remove(dest)?;
            state.files.remove(dest);
        }
        Kind::Keep => {
            state
                .files
                .insert(dest.clone(), action.hash.clone().context("missing hash")?);
        }
        Kind::Release | Kind::Forget => {
            state.files.remove(dest);
        }
        Kind::KeepEdited | Kind::Pending | Kind::SkipDeleted => {}
    }
    Ok(())
}

fn save_original(dir: &Path, record: &mut Backup, original: &Path) -> anyhow::Result<()> {
    let stored = Backup::stored(dir, original);
    fs::create_dir_all(stored.parent().context("backup path has no parent")?)?;
    // Read as the user: system files Ghost manages (themes, udev rules, SDDM
    // drop-ins) are world-readable, and the copy stays in the user's backups.
    fs::copy(original, &stored).with_context(|| format!("backing up {}", original.display()))?;
    record.saved.push(original.to_owned());
    Ok(())
}

fn new_backup_id(backups: &Path) -> anyhow::Result<String> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let base = timestamp(secs);
    let mut id = base.clone();
    let mut n = 1;
    while backups.join(&id).exists() || backups.join(format!("{id}.restored")).exists() {
        id = format!("{base}-{n}");
        n += 1;
    }
    Ok(id)
}

/// UTC `YYYYMMDDTHHMMSSZ` (civil-from-days, H. Hinnant).
fn timestamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}{month:02}{day:02}T{:02}{:02}{:02}Z",
        rem / 3_600,
        rem % 3_600 / 60,
        rem % 60
    )
}

fn private_state_dirs(paths: &Paths) -> anyhow::Result<()> {
    if let Some(dir) = paths.state.parent() {
        private_dir(dir)?;
    }
    private_dir(&paths.backups)
}

/// Backups that can still be restored, oldest first.
pub fn list_backups(paths: &Paths) -> anyhow::Result<Vec<Backup>> {
    let Ok(dir) = fs::read_dir(&paths.backups) else {
        return Ok(Vec::new());
    };
    let mut backups = Vec::new();
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".restored") {
            continue;
        }
        let index = entry.path().join("index.json");
        let text =
            fs::read_to_string(&index).with_context(|| format!("reading {}", index.display()))?;
        backups.push(
            serde_json::from_str::<Backup>(&text)
                .with_context(|| format!("parsing {}", index.display()))?,
        );
    }
    backups.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(backups)
}

#[derive(Debug, Default, PartialEq)]
pub struct RestoreReport {
    pub restored: Vec<PathBuf>,
    pub removed: Vec<PathBuf>,
    /// Created by Ghost but changed since; left in place.
    pub kept: Vec<PathBuf>,
}

/// Undoes the most recent deployment: originals come back, files Ghost created
/// are removed unless you changed them, and state returns to what it was.
/// Restoring repeatedly steps further back.
pub fn restore_latest(
    paths: &Paths,
    ops: &Ops,
    dry_run: bool,
) -> anyhow::Result<Option<(String, RestoreReport)>> {
    let Some(backup) = list_backups(paths)?.pop() else {
        return Ok(None);
    };
    let dir = paths.backups.join(&backup.id);
    if !dry_run {
        private_state_dirs(paths)?;
    }
    let mut report = RestoreReport::default();

    for (path, hash) in &backup.created {
        if backup.saved.contains(path) {
            continue;
        }
        match hash_file(path)? {
            Some(current) if current == *hash => {
                if !dry_run {
                    ops.remove(path)?;
                }
                report.removed.push(path.clone());
            }
            Some(_) => report.kept.push(path.clone()),
            None => {}
        }
    }
    for original in &backup.saved {
        if !dry_run {
            ops.create_parents(original)?;
            ops.install(&Backup::stored(&dir, original), original)?;
        }
        report.restored.push(original.clone());
    }
    if !dry_run {
        for created in backup.created_dirs.iter().rev() {
            // Only empty directories go; anything you added keeps its directory.
            ops.rmdir_if_empty(created);
        }
        backup.state_before.save(&paths.state)?;
        let mut done = dir.clone().into_os_string();
        done.push(".restored");
        fs::rename(&dir, done)?;
    }
    Ok(Some((backup.id, report)))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("target/test-fixtures/deploy")
                .join(name);
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("src")).unwrap();
            fs::create_dir_all(root.join("home")).unwrap();
            Self { root }
        }

        fn src(&self, name: &str, text: &str) -> PathBuf {
            let path = self.root.join("src").join(name);
            fs::write(&path, text).unwrap();
            path
        }

        fn dest(&self, rel: &str) -> PathBuf {
            self.root.join("home").join(rel)
        }

        fn paths(&self) -> Paths {
            Paths::under(&self.root.join("state"))
        }

        /// Direct writes only: tests never call sudo.
        fn ops(&self) -> Ops {
            Ops::new(&self.root.join("home"), false, Vec::new())
        }

        fn deploy(&self, entries: &[Entry]) -> (Plan, Option<String>) {
            let paths = self.paths();
            let mut state = State::load(&paths.state).unwrap();
            let plan = plan(entries, &state).unwrap();
            let id = execute(
                &plan,
                &mut state,
                &paths,
                &self.ops(),
                Some("rev".into()),
                vec!["core".into()],
            )
            .unwrap();
            (plan, id)
        }

        fn kind_of(plan: &Plan, dest: &Path) -> Kind {
            plan.actions.iter().find(|a| a.dest == dest).unwrap().kind
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn fresh_install_creates_files_and_dirs_then_restore_removes_them() {
        let fx = Fixture::new("fresh");
        let dest = fx.dest(".config/hypr/hyprland.lua");
        let entries = [Entry {
            src: fx.src("a", "v1"),
            dest: dest.clone(),
        }];

        let (plan, id) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &dest), Kind::Create);
        assert!(id.is_some());
        assert_eq!(read(&dest), "v1");

        let (plan, id) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &dest), Kind::Keep);
        assert!(id.is_none(), "an unchanged re-run makes no backup");

        let (_, report) = restore_latest(&fx.paths(), &fx.ops(), false)
            .unwrap()
            .unwrap();
        assert_eq!(report.removed, std::slice::from_ref(&dest));
        assert!(!dest.exists());
        assert!(
            !fx.dest(".config").exists(),
            "directories Ghost created are removed when empty"
        );
        let restored = State::load(&fx.paths().state).unwrap();
        assert!(restored.files.is_empty());
        assert!(
            restored.components.is_empty(),
            "components return to the pre-install list"
        );
        assert!(
            restore_latest(&fx.paths(), &fx.ops(), false)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn existing_untracked_file_is_backed_up_and_restored() {
        let fx = Fixture::new("replace");
        let dest = fx.dest("waybar.css");
        fs::write(&dest, "mine").unwrap();
        let entries = [Entry {
            src: fx.src("b", "ghost"),
            dest: dest.clone(),
        }];

        let (plan, _) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &dest), Kind::Replace);
        assert_eq!(read(&dest), "ghost");

        let (_, report) = restore_latest(&fx.paths(), &fx.ops(), false)
            .unwrap()
            .unwrap();
        assert_eq!(report.restored, std::slice::from_ref(&dest));
        assert_eq!(read(&dest), "mine");
    }

    #[test]
    fn three_way_update() {
        let fx = Fixture::new("three-way");
        let untouched = fx.dest("untouched");
        let edited = fx.dest("edited");
        let edited_same = fx.dest("edited-same");
        let src_untouched = fx.src("u", "v1");
        let src_edited = fx.src("e", "v1");
        let src_same = fx.src("s", "v1");
        let entries = [
            Entry {
                src: src_edited.clone(),
                dest: edited.clone(),
            },
            Entry {
                src: src_same.clone(),
                dest: edited_same.clone(),
            },
            Entry {
                src: src_untouched.clone(),
                dest: untouched.clone(),
            },
        ];
        fx.deploy(&entries);

        fs::write(&edited, "my edit").unwrap();
        fs::write(&edited_same, "my edit").unwrap();
        fs::write(&src_untouched, "v2").unwrap();
        fs::write(&src_edited, "v2").unwrap();

        let (plan, _) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &untouched), Kind::Update);
        assert_eq!(Fixture::kind_of(&plan, &edited), Kind::Conflict);
        assert_eq!(Fixture::kind_of(&plan, &edited_same), Kind::KeepEdited);
        assert_eq!(read(&untouched), "v2");
        assert_eq!(read(&edited), "my edit");
        assert_eq!(read(&ghost_new(&edited)), "v2");
        assert!(!ghost_new(&edited_same).exists());

        // Still recognised next time (state kept the hash Ghost originally
        // wrote), without rewriting the unreviewed .ghost-new.
        let state = State::load(&fx.paths().state).unwrap();
        assert_eq!(plan_kind(&entries, &state, &edited), Kind::Pending);
        fs::remove_file(ghost_new(&edited)).unwrap();
        assert_eq!(plan_kind(&entries, &state, &edited), Kind::Conflict);
        fs::write(ghost_new(&edited), "v2").unwrap();

        let (_, report) = restore_latest(&fx.paths(), &fx.ops(), false)
            .unwrap()
            .unwrap();
        assert_eq!(read(&untouched), "v1");
        assert!(report.removed.contains(&ghost_new(&edited)));
        assert_eq!(read(&edited), "my edit", "restore never touches your edits");
    }

    fn plan_kind(entries: &[Entry], state: &State, dest: &Path) -> Kind {
        let plan = plan(entries, state).unwrap();
        Fixture::kind_of(&plan, dest)
    }

    #[test]
    fn dropped_files_removed_only_if_unedited() {
        let fx = Fixture::new("dropped");
        let keep = fx.dest("keep");
        let gone = fx.dest("gone");
        let edited = fx.dest("edited");
        let all = [
            Entry {
                src: fx.src("k", "k"),
                dest: keep.clone(),
            },
            Entry {
                src: fx.src("g", "g"),
                dest: gone.clone(),
            },
            Entry {
                src: fx.src("e", "e"),
                dest: edited.clone(),
            },
        ];
        fx.deploy(&all);
        fs::write(&edited, "mine").unwrap();

        let (plan, _) = fx.deploy(&all[..1]);
        assert_eq!(Fixture::kind_of(&plan, &gone), Kind::Remove);
        assert_eq!(Fixture::kind_of(&plan, &edited), Kind::Release);
        assert!(!gone.exists());
        assert_eq!(read(&edited), "mine");
        let state = State::load(&fx.paths().state).unwrap();
        assert!(!state.files.contains_key(&edited) && !state.files.contains_key(&gone));

        restore_latest(&fx.paths(), &fx.ops(), false).unwrap();
        assert_eq!(read(&gone), "g");
    }

    #[test]
    fn deleted_by_user_is_not_recreated() {
        let fx = Fixture::new("deleted");
        let dest = fx.dest("x");
        let entries = [Entry {
            src: fx.src("x", "x"),
            dest: dest.clone(),
        }];
        fx.deploy(&entries);
        fs::remove_file(&dest).unwrap();
        let (plan, id) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &dest), Kind::SkipDeleted);
        assert!(id.is_none());
        assert!(!dest.exists());
    }

    #[test]
    fn symlinked_destination_is_never_replaced() {
        let fx = Fixture::new("symlink");
        let target = fx.root.join("dotfiles-style.css");
        fs::write(&target, "dotfiles").unwrap();
        let dest = fx.dest("style.css");
        std::os::unix::fs::symlink(&target, &dest).unwrap();
        let entries = [Entry {
            src: fx.src("s", "ghost"),
            dest: dest.clone(),
        }];
        let (plan, _) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &dest), Kind::Conflict);
        assert!(
            fs::symlink_metadata(&dest)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(read(&target), "dotfiles");
        assert_eq!(read(&ghost_new(&dest)), "ghost");

        let (plan, id) = fx.deploy(&entries);
        assert_eq!(Fixture::kind_of(&plan, &dest), Kind::Pending);
        assert!(id.is_none(), "an unreviewed .ghost-new is not rewritten");

        fs::write(&target, "ghost").unwrap();
        let (plan, _) = fx.deploy(&entries);
        assert_eq!(
            Fixture::kind_of(&plan, &dest),
            Kind::Keep,
            "a symlink already pointing at identical content is fine"
        );
    }

    #[test]
    fn executable_bit_is_preserved() {
        use std::os::unix::fs::PermissionsExt;
        let fx = Fixture::new("exec");
        let src = fx.src("media.sh", "#!/bin/sh\n");
        fs::set_permissions(&src, fs::Permissions::from_mode(0o755)).unwrap();
        let dest = fx.dest("scripts/media.sh");
        fx.deploy(&[Entry {
            src,
            dest: dest.clone(),
        }]);
        assert_eq!(
            fs::metadata(&dest).unwrap().permissions().mode() & 0o777,
            0o755
        );
    }

    #[test]
    fn restores_step_back_through_history() {
        let fx = Fixture::new("history");
        let dest = fx.dest("f");
        fs::write(&dest, "original").unwrap();
        let src = fx.src("f", "v1");
        let entries = [Entry {
            src: src.clone(),
            dest: dest.clone(),
        }];
        fx.deploy(&entries);
        fs::write(&src, "v2").unwrap();
        fx.deploy(&entries);
        assert_eq!(list_backups(&fx.paths()).unwrap().len(), 2);

        restore_latest(&fx.paths(), &fx.ops(), false).unwrap();
        assert_eq!(read(&dest), "v1");
        restore_latest(&fx.paths(), &fx.ops(), false).unwrap();
        assert_eq!(read(&dest), "original");
    }

    #[test]
    fn dry_run_restore_changes_nothing() {
        let fx = Fixture::new("dry");
        let dest = fx.dest("f");
        let entries = [Entry {
            src: fx.src("f", "x"),
            dest: dest.clone(),
        }];
        fx.deploy(&entries);
        let (_, report) = restore_latest(&fx.paths(), &fx.ops(), true)
            .unwrap()
            .unwrap();
        assert_eq!(report.removed, std::slice::from_ref(&dest));
        assert!(dest.exists());
        assert_eq!(list_backups(&fx.paths()).unwrap().len(), 1);
    }

    #[test]
    fn sudo_only_outside_home_and_only_when_enabled() {
        let home = Path::new("/home/u");
        let ops = Ops::new(home, true, Vec::new());
        assert!(!ops.needs_root(Path::new("/home/u/.config/hypr/hyprland.lua")));
        assert!(ops.needs_root(Path::new("/etc/sddm.conf.d/zz-ghost.conf")));
        assert!(
            ops.needs_root(Path::new("/home/user2/x")),
            "another home is not ours"
        );
        assert!(!Ops::new(home, false, Vec::new()).needs_root(Path::new("/etc/x")));
    }

    #[test]
    fn sudo_confined_to_root_destinations() {
        let ops = Ops::new(
            Path::new("/home/u"),
            true,
            vec![
                PathBuf::from("/usr/share/sddm/themes/ghost"),
                PathBuf::from("/etc/udev/rules.d/61-ghost-nvidia-dgpu.rules"),
            ],
        );
        let ok = |p: &str, r| ops.check_root(Path::new(p), r).is_ok();
        assert!(ok("/usr/share/sddm/themes/ghost/Main.qml", Reach::Inside));
        assert!(ok(
            "/etc/udev/rules.d/61-ghost-nvidia-dgpu.rules",
            Reach::Inside
        ));
        assert!(!ok("/etc/udev/rules.d/99-evil.rules", Reach::Inside));
        assert!(!ok("/etc/sudoers.d/x", Reach::Inside));
        assert!(!ok(
            "/usr/share/sddm/themes/ghost/../../../../etc/passwd",
            Reach::Inside
        ));
        assert!(!ok("relative/path", Reach::Inside));
        // Parents may be created for a destination, but not removed or written.
        assert!(ok("/usr/share/sddm/themes", Reach::InsideOrAncestor));
        assert!(!ok("/usr/share/sddm/themes", Reach::Inside));
        assert!(!ok("/usr/lib", Reach::InsideOrAncestor));
    }

    #[test]
    fn installed_files_never_group_or_world_writable() {
        use std::os::unix::fs::PermissionsExt;
        let fx = Fixture::new("modes");
        let loose = fx.src("loose", "x");
        fs::set_permissions(&loose, fs::Permissions::from_mode(0o777)).unwrap();
        let dest = fx.dest("loose");
        fx.deploy(&[Entry {
            src: loose,
            dest: dest.clone(),
        }]);
        assert_eq!(
            fs::metadata(&dest).unwrap().permissions().mode() & 0o7777,
            0o755
        );
    }

    #[test]
    fn state_and_backups_are_private() {
        use std::os::unix::fs::PermissionsExt;
        let fx = Fixture::new("private");
        fx.deploy(&[Entry {
            src: fx.src("f", "x"),
            dest: fx.dest("f"),
        }]);
        let paths = fx.paths();
        for dir in [paths.state.parent().unwrap(), paths.backups.as_path()] {
            assert_eq!(
                fs::metadata(dir).unwrap().permissions().mode() & 0o777,
                0o700,
                "{}",
                dir.display()
            );
        }
    }

    #[test]
    fn timestamp_format() {
        assert_eq!(timestamp(0), "19700101T000000Z");
        assert_eq!(timestamp(1_788_566_400), "20260905T000000Z");
        assert_eq!(timestamp(951_782_400 + 3_661), "20000229T010101Z");
    }
}
