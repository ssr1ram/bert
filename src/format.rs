// Task format detection and normalization
//
// bert reads every reasonable task-file convention leniently and commits to a
// single convention when writing. The convention comes from three sources, in
// precedence order:
//
//   1. explicit `format:` section in `.bert/config.yml` / legacy `skills.yml`
//   2. mimicry of the existing tasks directory (the directory is the config)
//   3. bert-native defaults
//
// This module implements parsing, detection (layer 2 helpers) and the status
// vocabulary normalization shared by all layers.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Canonical internal statuses used for filtering and display.
///
/// Raw strings on disk are never rewritten; they are only mapped onto this
/// vocabulary so commands can reason uniformly across formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Not started ("open", "pending", "stub", "new", ...)
    Todo,
    /// In flight ("in-progress", "active", "wip", ...)
    Doing,
    /// Finished ("done", "completed", "closed", ...)
    Done,
    /// Cannot proceed ("blocked", "waiting", ...)
    Blocked,
    /// Deliberately not being worked on ("paused", "deferred", "on-hold", ...)
    Parked,
}

impl Status {
    /// Canonical name; used by filtering/listing (upcoming).
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::Doing => "doing",
            Status::Done => "done",
            Status::Blocked => "blocked",
            Status::Parked => "parked",
        }
    }
}

/// Synonym table mapping raw status words onto [`Status`].
///
/// Comparison is case-insensitive; unknown words normalize to `None` and pass
/// through untouched anywhere statuses are echoed back to the user.
const STATUS_SYNONYMS: &[(&str, Status)] = &[
    ("open", Status::Todo),
    ("new", Status::Todo),
    ("stub", Status::Todo),
    ("pending", Status::Todo),
    ("todo", Status::Todo),
    ("to-do", Status::Todo),
    ("in-progress", Status::Doing),
    ("in_progress", Status::Doing),
    ("active", Status::Doing),
    ("wip", Status::Doing),
    ("doing", Status::Doing),
    ("done", Status::Done),
    ("completed", Status::Done),
    ("complete", Status::Done),
    ("closed", Status::Done),
    ("fixed", Status::Done),
    ("blocked", Status::Blocked),
    ("waiting", Status::Blocked),
    ("paused", Status::Parked),
    ("parked", Status::Parked),
    ("deferred", Status::Parked),
    ("on-hold", Status::Parked),
    ("hold", Status::Parked),
    ("backlog", Status::Parked),
];

/// Map a raw status string onto the canonical vocabulary.
pub fn normalize_status(raw: &str) -> Option<Status> {
    let needle = raw.trim().to_lowercase();
    STATUS_SYNONYMS
        .iter()
        .find(|(word, _)| *word == needle)
        .map(|(_, status)| *status)
}

/// A parsed task filename: number plus optional slug.
///
/// Accepts every observed convention:
/// - `task-01-slug.md`     (bert native)
/// - `task-08.1-child.md`  (bert subtask)
/// - `task-013.md`         (bare, zero-padded — e.g. LLM-generated dirs)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskFilename {
    /// The number exactly as written in the filename (padding preserved).
    pub number: String,
    /// Slug portion if present.
    pub slug: Option<String>,
}

/// Parse a task filename; returns `None` for non-task files or malformed names.
pub fn parse_task_filename(filename: &str) -> Option<TaskFilename> {
    let rest = filename.strip_prefix("task-")?;
    let rest = rest.strip_suffix(".md")?;
    let (number, slug) = match rest.split_once('-') {
        Some((num, slug)) => (num, Some(slug.to_string())),
        None => (rest, None),
    };

    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }

    Some(TaskFilename {
        number: number.to_string(),
        slug,
    })
}

/// Compare two task numbers segment-wise by numeric value, so `"1"` matches
/// `"01"` and `"8.1"` matches `"08.01"`.
pub fn number_matches(a: &str, b: &str) -> bool {
    let mut ia = a.split('.');
    let mut ib = b.split('.');
    loop {
        match (ia.next(), ib.next()) {
            (Some(x), Some(y)) => {
                let (x, y) = (x.parse::<u64>(), y.parse::<u64>());
                if x.is_err() || y.is_err() || x.unwrap() != y.unwrap() {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

/// True when `number` is a strict descendant of `parent` (e.g. `08.1` under `08`).
pub fn number_is_descendant(number: &str, parent: &str) -> bool {
    let segs: Vec<&str> = number.split('.').collect();
    let psegs = parent.split('.').count();
    segs.len() > psegs && number_matches(&segs[..psegs].join("."), parent)
}

/// How a frontmatter field was observed to be valued, driving placeholders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Scalar,
    List,
}

/// The write-convention inferred from an existing tasks directory.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatProfile {
    /// Emit `task-NN-slug.md` (true, bert native) or bare `task-NNN.md`.
    pub use_slug: bool,
    /// Digit width for top-level numbers (max observed, floor of 2).
    pub number_width: usize,
    /// Frontmatter keys emitted on new stubs, in observed order.
    pub frontmatter_keys: Vec<(String, FieldKind)>,
    /// H1 style: `# task-NNN: Title` (true) vs `# Task NN: Title`.
    pub h1_lowercase: bool,
    /// The raw word this directory uses for a fresh/todo task.
    pub todo_status_word: String,
    /// Distinct raw status words observed (for `adopt` persistence).
    pub observed_statuses: Vec<String>,
}

impl Default for FormatProfile {
    fn default() -> Self {
        Self {
            use_slug: true,
            number_width: 2,
            frontmatter_keys: vec![
                ("status".to_string(), FieldKind::Scalar),
                ("created".to_string(), FieldKind::Scalar),
            ],
            h1_lowercase: false,
            todo_status_word: "stub".to_string(),
            observed_statuses: Vec::new(),
        }
    }
}

impl FormatProfile {
    /// Pad a top-level number to the profile's width.
    #[allow(dead_code)]
    pub fn pad(&self, number: &str) -> String {
        let mut segs: Vec<String> = number.split('.').map(|s| s.to_string()).collect();
        segs[0] = zero_pad(&segs[0], self.number_width);
        segs.join(".")
    }
}

/// Left-pad a digit string with zeros to at least `width` characters.
///
/// (`format!("{:03}", s)` space-pads strings rather than zero-padding, so
/// this is done manually.)
fn zero_pad(digits: &str, width: usize) -> String {
    let mut out = String::from(digits);
    while out.len() < width {
        out.insert(0, '0');
    }
    out
}

/// Overlay an explicit config declaration onto a detected/default profile.
///
/// Precedence: explicit `format:` section > directory mimicry > defaults.
pub fn apply_overrides(
    mut profile: FormatProfile,
    cfg: Option<&crate::models::config::FileFormatConfig>,
) -> FormatProfile {
    let Some(cfg) = cfg else {
        return profile;
    };
    if let Some(slug) = cfg.slug {
        profile.use_slug = slug;
    }
    if let Some(padding) = cfg.padding {
        profile.number_width = padding.max(2);
    }
    if let Some(lower) = cfg.h1_lowercase {
        profile.h1_lowercase = lower;
    }
    if let Some(word) = &cfg.todo_status_word {
        profile.todo_status_word = word.clone();
    }
    if let Some(keys) = &cfg.frontmatter {
        // Preserve detected value kinds (scalar vs list) where known.
        profile.frontmatter_keys = keys
            .iter()
            .map(|k| {
                let kind = profile
                    .frontmatter_keys
                    .iter()
                    .find(|(name, _)| name == k)
                    .map(|(_, kind)| *kind)
                    .unwrap_or(FieldKind::Scalar);
                (k.clone(), kind)
            })
            .collect();
    }
    profile
}

/// Infer the directory's write-convention from its existing files.
///
/// Deterministic tie-breaks: fractions are compared with a >50% bar, ties
/// resolve to the simpler shape (no slug, uppercase H1), and an empty or
/// missing directory yields the bert-native default.
/// Walk `tasks_dir` once, gathering the format profile and the count of
/// parseable task files (including subtasks).
pub fn analyze_tasks(tasks_dir: &Path) -> (FormatProfile, usize) {
    let mut profile = FormatProfile::default();
    let mut parsed_count = 0usize;

    let entries = match std::fs::read_dir(tasks_dir) {
        Ok(entries) => entries,
        Err(_) => return (profile, parsed_count),
    };

    let mut slugged = 0usize;
    let mut bare = 0usize;
    let mut max_width = 2usize;
    let mut fm_key_counts: BTreeMap<String, (usize, FieldKind)> = BTreeMap::new();
    let mut fm_file_count = 0usize;
    let mut h1_lower = 0usize;
    let mut h1_upper = 0usize;
    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut key_order: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(parsed) = parse_task_filename(name) else {
            continue;
        };
        parsed_count += 1;

        // Filename shape statistics (top-level only drives width/slug).
        if !parsed.number.contains('.') {
            if parsed.slug.as_deref().map(str::trim).unwrap_or("").is_empty() {
                bare += 1;
            } else {
                slugged += 1;
            }
            max_width = max_width.max(parsed.number.len());
        }

        // File content statistics.
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(fm) = extract_frontmatter(&content) else {
            continue;
        };
        fm_file_count += 1;

        if let serde_yaml::Value::Mapping(map) = &fm {
            for key in map.keys() {
                if let Some(k) = key.as_str() {
                    if !key_order.iter().any(|existing| existing == k) {
                        key_order.push(k.to_string());
                    }
                    let entry = fm_key_counts
                        .entry(k.to_string())
                        .or_insert((0, FieldKind::Scalar));
                    entry.0 += 1;
                    if matches!(map.get(key), Some(serde_yaml::Value::Sequence(_))) {
                        entry.1 = FieldKind::List;
                    }
                }
            }

            // Status vocabulary.
            if let Some(serde_yaml::Value::String(status)) = map.get("status").cloned() {
                *status_counts.entry(status.clone()).or_insert(0) += 1;
            }
        }

        // H1 style.
        if let Some(h1) = content.lines().find(|l| l.starts_with("# ")) {
            if h1[2..].starts_with("task-") {
                h1_lower += 1;
            } else {
                h1_upper += 1;
            }
        }
    }

    let total_files = slugged + bare;
    if total_files == 0 {
        return (profile, parsed_count);
    }

    profile.use_slug = slugged * 2 > total_files;
    profile.number_width = max_width;
    profile.h1_lowercase = h1_lower > h1_upper;

    if fm_file_count > 0 {
        profile.frontmatter_keys = key_order
            .into_iter()
            .filter_map(|k| {
                fm_key_counts
                    .get(&k)
                    .filter(|(count, _)| count * 2 >= fm_file_count)
                    .map(|(_, kind)| (k, *kind))
            })
            .collect();
    }

    // The directory's word for a fresh task: most common raw status mapping
    // to Todo; fall back through synonyms to bert's own word.
    profile.observed_statuses = status_counts.keys().cloned().collect();
    profile.todo_status_word = status_counts
        .iter()
        .filter(|(raw, _)| normalize_status(raw) == Some(Status::Todo))
        .max_by_key(|(_, count)| **count)
        .map(|(raw, _)| raw.clone())
        .unwrap_or_else(|| profile.todo_status_word.clone());

    (profile, parsed_count)
}

/// Detect the directory's write conventions.
///
/// Thin wrapper over [`analyze_tasks`] for callers that only need the profile.
pub fn detect_profile(tasks_dir: &Path) -> FormatProfile {
    analyze_tasks(tasks_dir).0
}

/// Extract the YAML frontmatter block of a markdown file, if any.
fn extract_frontmatter(content: &str) -> Option<serde_yaml::Value> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let block = &rest[..end];
    serde_yaml::from_str(block).ok()
}

/// Render the frontmatter for a new stub, mimicking the detected profile.
///
/// Keys the directory uses get derived values where possible (`status`,
/// `created`, `updated`, `id`, `title`) and typed empty placeholders
/// otherwise (`track: ""`, `tags: []`).
pub fn render_frontmatter(profile: &FormatProfile, padded_number: &str, description: &str) -> String {
    use chrono::Local;
    let today = Local::now().format("%Y-%m-%d");

    let mut lines: Vec<String> = Vec::new();
    for (key, kind) in &profile.frontmatter_keys {
        let line = match key.as_str() {
            "status" => format!("status: {}", profile.todo_status_word),
            "created" | "updated" => format!("{}: {}", key, today),
            "id" => format!("id: task-{}", padded_number),
            "title" => format!("title: \"{}\"", description.replace('"', "'")),
            k if *kind == FieldKind::List => format!("{}: []", k),
            k => format!("{}: \"\"", k),
        };
        lines.push(line);
    }

    if lines.is_empty() {
        return String::new();
    }
    format!("---\n{}\n---\n\n", lines.join("\n"))
}

/// One parsed task file, ready for listing and filtering.
///
/// Assembled leniently: every field is best-effort across conventions
/// (frontmatter title, H1 fallback, raw status strings, any padding).
#[derive(Debug, Clone, Serialize)]
pub struct TaskEntry {
    /// Task number exactly as written in the filename (padding preserved).
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    pub title: String,
    /// Raw status string from frontmatter (empty when the file has none).
    pub status: String,
    /// Canonical status, when the raw word is recognized.
    pub canonical_status: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
    #[serde(skip)]
    #[allow(dead_code)]
    pub path: PathBuf,
}

impl TaskEntry {
    /// Nesting depth: 0 for top-level tasks, 1 for `08.1`, etc.
    pub fn depth(&self) -> usize {
        self.number.split('.').count().saturating_sub(1)
    }
}

/// Scan a tasks directory into structured entries, sorted numerically.
///
/// Unreadable or unparseable files are skipped silently; `README.md` and
/// other non-task files never match the task-filename pattern anyway.
pub fn scan_task_entries(tasks_dir: &Path) -> Vec<TaskEntry> {
    let Ok(entries) = std::fs::read_dir(tasks_dir) else {
        return Vec::new();
    };

    let mut tasks: Vec<TaskEntry> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            let filename = path.file_name()?.to_str()?;
            let parsed = parse_task_filename(filename)?;

            let content = std::fs::read_to_string(&path).ok()?;
            let fm = extract_frontmatter(&content);
            let mapping = fm.as_ref().and_then(|v| v.as_mapping());

            let string_field = |key: &str| -> Option<String> {
                mapping
                    .and_then(|m| m.get(key))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            };
            let seq_field = |key: &str| -> Vec<String> {
                mapping
                    .and_then(|m| m.get(key))
                    .and_then(|v| v.as_sequence())
                    .map(|seq| {
                        seq.iter()
                            .filter_map(|item| item.as_str())
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default()
            };

            let status_raw = string_field("status").unwrap_or_default();

            // Title: frontmatter → H1 (minus "task-NNN:"/"Task NN:" prefix)
            //        → slug → "untitled"
            let h1 = content
                .lines()
                .find(|l| l.starts_with("# "))
                .map(|l| l[2..].trim().to_string());
            let title = string_field("title")
                .or_else(|| {
                    h1.clone().and_then(|line| {
                        line.split_once(':').and_then(|(prefix, rest)| {
                            prefix.to_lowercase().contains("task")
                                .then(|| rest.trim().to_string())
                                .filter(|t| !t.is_empty())
                        })
                    })
                })
                .or(parsed.slug.clone())
                .unwrap_or_else(|| "untitled".to_string());

            Some(TaskEntry {
                slug: parsed.slug,
                title,
                canonical_status: normalize_status(&status_raw),
                track: string_field("track"),
                priority: string_field("priority"),
                created: string_field("created"),
                updated: string_field("updated"),
                tags: seq_field("tags"),
                references: seq_field("references"),
                number: parsed.number,
                status: status_raw,
                path,
            })
        })
        .collect();

    tasks.sort_by_cached_key(|t| number_key(&t.number));
    tasks
}

/// Numeric sort key for a task number: `"08.2"` → `[8, 2]`.
fn number_key(number: &str) -> Vec<u64> {
    number
        .split('.')
        .map(|seg| seg.parse::<u64>().unwrap_or(0))
        .collect()
}

/// Does an entry's raw status match a user-supplied filter?
///
/// Synonyms are equated via [`normalize_status`] (`parked` matches files
/// saying `paused` or `deferred`); unrecognized words fall back to exact
/// case-insensitive comparison so custom vocabularies stay filterable.
pub fn status_matches(entry_status: &str, filter: &str) -> bool {
    match (normalize_status(filter), normalize_status(entry_status)) {
        (Some(want), Some(have)) => want == have,
        _ => entry_status.eq_ignore_ascii_case(filter),
    }
}

/// Simple field equality filter (case-insensitive), e.g. for track/priority.
pub fn field_matches(value: &Option<String>, filter: &str) -> bool {
    value
        .as_deref()
        .map(|v| v.eq_ignore_ascii_case(filter))
        .unwrap_or(false)
}

/// Tag membership filter (case-insensitive).
pub fn has_tag(entry: &TaskEntry, tag: &str) -> bool {
    entry.tags.iter().any(|t| t.eq_ignore_ascii_case(tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bert_native() {
        let parsed = parse_task_filename("task-01-smoke-test.md").unwrap();
        assert_eq!(parsed.number, "01");
        assert_eq!(parsed.slug.as_deref(), Some("smoke-test"));
    }

    #[test]
    fn test_parse_subtask() {
        let parsed = parse_task_filename("task-08.1-child.md").unwrap();
        assert_eq!(parsed.number, "08.1");
        assert_eq!(parsed.slug.as_deref(), Some("child"));
    }

    #[test]
    fn test_parse_bare_wobase_style() {
        let parsed = parse_task_filename("task-013.md").unwrap();
        assert_eq!(parsed.number, "013");
        assert_eq!(parsed.slug, None);
    }

    #[test]
    fn test_parse_rejects_non_tasks_and_garbage() {
        assert!(parse_task_filename("README.md").is_none());
        assert!(parse_task_filename("task-.md").is_none());
        assert!(parse_task_filename("task-abc-def.md").is_none());
        assert!(parse_task_filename("note-01-x.md").is_none());
    }

    #[test]
    fn test_number_matches_across_padding() {
        assert!(number_matches("1", "01"));
        assert!(number_matches("013", "13"));
        assert!(number_matches("8.1", "08.01"));
        assert!(!number_matches("1", "2"));
        assert!(!number_matches("1", "1.1"));
    }

    #[test]
    fn test_descendants() {
        assert!(number_is_descendant("08.1", "08"));
        assert!(number_is_descendant("08.1.2", "08"));
        assert!(number_is_descendant("08.1.2", "08.1"));
        assert!(!number_is_descendant("08", "08"));
        assert!(!number_is_descendant("09", "08"));
    }

    #[test]
    fn test_status_synonyms() {
        assert_eq!(normalize_status("open"), Some(Status::Todo));
        assert_eq!(normalize_status("Paused"), Some(Status::Parked));
        assert_eq!(normalize_status("completed"), Some(Status::Done));
        assert_eq!(normalize_status("in-progress"), Some(Status::Doing));
        assert_eq!(normalize_status("shipped"), None);
    }

    #[test]
    fn test_scan_task_entries_wobase_fixture() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        std::fs::write(
            dir.join("task-013.md"),
            "---\nid: task-013\ntitle: \"Resend dashboard setup\"\nstatus: paused\ntrack: newsletter\npriority: p1\ntags:\n  - resend\nreferences:\n  - docs/dev/newsletter-resend.md\n---\n\n# task-013: Resend dashboard setup\n",
        ).unwrap();
        std::fs::write(
            dir.join("task-002.md"),
            "---\nid: task-002\ntitle: \"Hard-paywall excludes\"\nstatus: open\ntrack: wire\npriority: p1\n---\n",
        ).unwrap();
        std::fs::write(dir.join("README.md"), "# index").unwrap();

        let entries = scan_task_entries(dir);
        assert_eq!(entries.len(), 2, "README must not be scanned");

        // Sorted numerically despite 3-digit padding
        assert_eq!(entries[0].number, "002");
        assert_eq!(entries[0].canonical_status, Some(Status::Todo));
        assert_eq!(entries[1].number, "013");
        assert_eq!(entries[1].title, "Resend dashboard setup");
        assert_eq!(entries[1].canonical_status, Some(Status::Parked));
        assert_eq!(entries[1].track.as_deref(), Some("newsletter"));
        assert_eq!(entries[1].tags, vec!["resend"]);
        assert_eq!(entries[1].references.len(), 1);
    }

    #[test]
    fn test_scan_title_falls_back_to_h1_then_slug() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        // H1 only (bert-native stub shape)
        std::fs::write(
            dir.join("task-01-alpha.md"),
            "---\nstatus: pending\n---\n\n# Task 01: Alpha task\n",
        ).unwrap();
        // No frontmatter at all
        std::fs::write(dir.join("task-02-beta.md"), "# task-002: Beta thing\n").unwrap();
        // Slug fallback (no frontmatter title, no usable H1)
        std::fs::write(dir.join("task-03-gamma-delta.md"), "just some text\n").unwrap();
        // Nothing usable at all
        std::fs::write(dir.join("task-4.md"), "just some text\n").unwrap();

        let entries = scan_task_entries(dir);
        assert_eq!(entries[0].title, "Alpha task");
        assert_eq!(entries[1].title, "Beta thing");
        assert_eq!(entries[2].title, "gamma-delta"); // slug fallback
        assert_eq!(entries[3].title, "untitled");
    }

    #[test]
    fn test_scan_sorts_mixed_padding_numerically() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        for name in ["task-10-x.md", "task-9-y.md", "task-100-z.md", "task-8-a.md", "task-09.5-sub.md"] {
            std::fs::write(dir.join(name), "").unwrap();
        }
        let numbers: Vec<String> = scan_task_entries(dir)
            .into_iter()
            .map(|e| e.number)
            .collect();
        assert_eq!(numbers, vec!["8", "9", "09.5", "10", "100"]);
    }

    #[test]
    fn test_status_filter_matches_synonyms() {
        assert!(status_matches("open", "todo"));
        assert!(status_matches("paused", "parked"));
        assert!(status_matches("deferred", "PARKED"));
        assert!(!status_matches("done", "doing"));
    }

    #[test]
    fn test_detect_wobase_style_directory() {
        let tmp = tempfile::TempDir::new().unwrap();
        let dir = tmp.path();
        for n in ["001", "002"] {
            std::fs::write(
                dir.join(format!("task-{}.md", n)),
                format!(
                    "---\nid: task-{}\ntitle: \"T{}\"\nstatus: open\npriority: p1\ntrack: wire\ntags:\n  - a\n---\n\n# task-{}: T{}\n",
                    n, n, n, n
                ),
            )
            .unwrap();
        }

        let profile = detect_profile(dir);
        assert!(!profile.use_slug);
        assert_eq!(profile.number_width, 3);
        assert!(profile.h1_lowercase);
        assert_eq!(profile.todo_status_word, "open");
        let keys: Vec<_> = profile.frontmatter_keys.iter().map(|(k, _)| k.clone()).collect();
        for expected in ["id", "title", "status", "priority", "track", "tags"] {
            assert!(keys.contains(&expected.to_string()), "missing {}", expected);
        }
        let tags_kind = profile
            .frontmatter_keys
            .iter()
            .find(|(k, _)| k == "tags")
            .unwrap()
            .1;
        assert_eq!(tags_kind, FieldKind::List);
    }

    #[test]
    fn test_detect_empty_dir_yields_default() {
        let tmp = tempfile::TempDir::new().unwrap();
        assert_eq!(detect_profile(tmp.path()), FormatProfile::default());
        assert_eq!(detect_profile(tmp.path().join("missing").as_path()), FormatProfile::default());
    }

    #[test]
    fn test_render_frontmatter_mimics_keys() {
        let profile = FormatProfile {
            use_slug: false,
            number_width: 3,
            frontmatter_keys: vec![
                ("id".into(), FieldKind::Scalar),
                ("title".into(), FieldKind::Scalar),
                ("status".into(), FieldKind::Scalar),
                ("track".into(), FieldKind::Scalar),
                ("tags".into(), FieldKind::List),
                ("created".into(), FieldKind::Scalar),
            ],
            ..Default::default()
        };
        let fm = render_frontmatter(&profile, "034", "Do a thing");
        assert!(fm.contains("id: task-034"));
        assert!(fm.contains("title: \"Do a thing\""));
        assert!(fm.contains("track: \"\""));
        assert!(fm.contains("tags: []"));
        assert!(fm.contains("created: "));
        assert!(!fm.contains("\"\"\""));
    }

    #[test]
    fn test_pad_respects_width() {
        let profile = FormatProfile { number_width: 3, ..Default::default() };
        assert_eq!(profile.pad("34"), "034");
        assert_eq!(profile.pad("8.1"), "008.1");
        let two = FormatProfile::default();
        assert_eq!(two.pad("7"), "07");
    }
}
