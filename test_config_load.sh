#!/bin/bash
# Test script to verify config loading

cat > /tmp/test_bert_config.rs << 'RUST_EOF'
// Quick test to verify bert config loading
use std::path::Path;

#[path = "src/errors.rs"]
mod errors;

#[path = "src/models/config.rs"]
mod config;

#[path = "src/project.rs"]
mod project;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let project_root = std::env::current_dir()?;
    let skill_yml_path = project_root.join(".claude/skills/bert/skill.yml");

    println!("Reading: {}", skill_yml_path.display());

    let yaml_content = fs::read_to_string(&skill_yml_path)?;

    let skill_config: config::SkillConfig = serde_yaml::from_str(&yaml_content)?;

    println!("\nDirectoryConfig deserialized:");
    println!("  library_directory: {:?}", skill_config.config.library_directory);
    println!("  sets_directory: {:?}", skill_config.config.sets_directory);

    let bert_config = config::BertConfig::from_skill_config(skill_config, &project_root);

    println!("\nBertConfig after conversion:");
    println!("  library_directory: {:?}", bert_config.library_directory);
    println!("  sets_directory: {:?}", bert_config.sets_directory);

    Ok(())
}
RUST_EOF

echo "Compiling test..."
rustc /tmp/test_bert_config.rs \
    --edition 2021 \
    --extern serde=/tmp/dummy \
    -L dependency=target/release/deps \
    -o /tmp/test_bert_config 2>&1 | head -20 || echo "Compilation needs dependencies"
