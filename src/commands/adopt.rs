// Task adopt command implementation
//
// `bert task adopt` scans the tasks directory, detects its conventions and
// persists them as an explicit `format:` section in `.bert/config.yml`, so
// bert's writing behavior is stable and reviewable even before/without
// files to mimic.
use crate::errors::{BertError, Result};
use crate::format;
use crate::models::config::BertConfig;
use crate::project::{BERT_CONFIG_DIR, BERT_CONFIG_FILE, LEGACY_SKILLS_YML};
use std::fs;
use std::path::Path;

/// Detect the tasks directory's format and write it to `.bert/config.yml`.
///
/// If only a legacy `skills.yml` exists, its contents are carried over into
/// the modern file so no configuration is lost when precedence shifts.
/// Returns a human-readable summary of what was adopted.
pub fn adopt(config: &BertConfig) -> Result<String> {
    let (profile, parsed_count) = format::analyze_tasks(&config.tasks_directory);

    if parsed_count == 0 {
        return Err(BertError::InvalidInput(format!(
            "no task files found in {} — nothing to adopt",
            config.tasks_directory.display()
        )));
    }

    let root = &config.project_root;
    let bert_dir = root.join(BERT_CONFIG_DIR);
    fs::create_dir_all(&bert_dir)?;
    let config_path = bert_dir.join(BERT_CONFIG_FILE);

    // Base document: existing modern config wins over a legacy copy.
    let base_text = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        let legacy = Path::new(LEGACY_SKILLS_YML);
        let legacy_path = root.join(legacy);
        if legacy_path.exists() {
            let mut text = fs::read_to_string(&legacy_path)?;
            text.push('\n');
            text
        } else {
            String::new()
        }
    };

    let mut doc: serde_yaml::Value = if base_text.trim().is_empty() {
        serde_yaml::Value::Mapping(serde_yaml::Mapping::new())
    } else {
        serde_yaml::from_str(&base_text).map_err(|e| {
            BertError::ConfigError(format!("existing config is not valid YAML: {}", e))
        })?
    };
    if !doc.is_mapping() {
        return Err(BertError::ConfigError(
            "existing config does not start with a mapping".to_string(),
        ));
    }

    // Build the format mapping from the detected profile.
    use serde_yaml::Value;
    let mut fmt = serde_yaml::Mapping::new();
    fmt.insert(Value::String("slug".into()), Value::Bool(profile.use_slug));
    fmt.insert(
        Value::String("padding".into()),
        Value::Number(profile.number_width.into()),
    );
    fmt.insert(
        Value::String("h1_lowercase".into()),
        Value::Bool(profile.h1_lowercase),
    );
    fmt.insert(
        Value::String("todo_status_word".into()),
        Value::String(profile.todo_status_word.clone()),
    );
    let keys: Vec<Value> = profile
        .frontmatter_keys
        .iter()
        .map(|(k, _)| Value::String(k.clone()))
        .collect();
    fmt.insert(Value::String("frontmatter".into()), Value::Sequence(keys));
    doc.as_mapping_mut()
        .unwrap()
        .insert(Value::String("format".into()), Value::Mapping(fmt));

    let out = serde_yaml::to_string(&doc)
        .map_err(|e| BertError::ConfigError(format!("failed to serialize config: {}", e)))?;
    fs::write(&config_path, out)?;

    Ok(format!(
        "adopted {} conventions: slug={} padding={} h1_lowercase={} status_word=\"{}\" frontmatter=[{}]\n  written to {}",
        config.tasks_directory.display(),
        profile.use_slug,
        profile.number_width,
        profile.h1_lowercase,
        profile.todo_status_word,
        profile
            .frontmatter_keys
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        config_path.display()
    ))
}
