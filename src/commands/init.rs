use anyhow::Result;
use std::fs;
use std::io::IsTerminal;
use std::path::Path;

use console::style;
use dialoguer::MultiSelect;

use crate::db;

const SKILL_MD: &str = include_str!("../../templates/SKILL.md");

struct SkillTarget {
    name: &'static str,
    /// Path relative to project root where SKILL.md is installed
    skill_path: &'static str,
    /// Paths to check for detecting the agent in the project
    detect: &'static [&'static str],
}

const TARGETS: &[SkillTarget] = &[
    SkillTarget {
        name: "Claude Code",
        skill_path: ".claude/skills/engram",
        detect: &[".claude", "CLAUDE.md"],
    },
    SkillTarget {
        name: "Cursor",
        skill_path: ".cursor/skills/engram",
        detect: &[".cursor", ".cursorrules"],
    },
    SkillTarget {
        name: "GitHub Copilot",
        skill_path: ".github/skills/engram",
        detect: &[".github"],
    },
    SkillTarget {
        name: "Windsurf",
        skill_path: ".windsurf/skills/engram",
        detect: &[".windsurf", ".windsurfrules"],
    },
    SkillTarget {
        name: "Roo Code",
        skill_path: ".roo/skills/engram",
        detect: &[".roo", ".clinerules"],
    },
    SkillTarget {
        name: "Gemini CLI",
        skill_path: ".gemini/skills/engram",
        detect: &[".gemini", "GEMINI.md"],
    },
    SkillTarget {
        name: "Universal (.agents) — Codex, and other compatible agents",
        skill_path: ".agents/skills/engram",
        detect: &[".agents", ".codex", "AGENTS.md"],
    },
];

fn is_detected(target: &SkillTarget, path: &Path) -> bool {
    target.detect.iter().any(|d| path.join(d).exists())
}

fn is_installed(target: &SkillTarget, path: &Path) -> bool {
    path.join(target.skill_path).join("SKILL.md").is_file()
}

pub fn run(path: &Path) -> Result<()> {
    let engram_dir = path.join(".engram");

    if engram_dir.exists() {
        println!(
            "{} .engram/ already exists, skipping initialization.",
            style("Note:").yellow()
        );
    } else {
        // Create directory structure
        let nodes_dir = engram_dir.join("nodes");
        let data_lake_dir = engram_dir.join("data_lake");
        fs::create_dir_all(&nodes_dir)?;
        fs::create_dir_all(&data_lake_dir)?;

        // Create .gitignore
        fs::write(engram_dir.join(".gitignore"), "engram.db\n")?;

        // Create top-level _index.yaml
        fs::write(nodes_dir.join("_index.yaml"), "namespaces: []\n")?;

        // Create SQLite DB with tables
        let conn = db::open(&engram_dir)?;
        db::create_tables(&conn)?;

        println!("{}", style("Initialized .engram/").green().bold());
    }

    // Build labels: mark detected and already installed agents
    let labels: Vec<String> = TARGETS
        .iter()
        .map(|t| {
            let mut label = t.name.to_string();
            if is_installed(t, path) {
                label.push_str(" (installed)");
            } else if is_detected(t, path) {
                label.push_str(" (detected)");
            }
            label
        })
        .collect();

    // Pre-select detected agents that aren't already installed
    let defaults: Vec<bool> = TARGETS
        .iter()
        .map(|t| !is_installed(t, path) && is_detected(t, path))
        .collect();

    let chosen: Vec<usize> = if std::io::stdin().is_terminal() {
        println!();
        MultiSelect::new()
            .with_prompt("Install engram skill for")
            .items(&labels)
            .defaults(&defaults)
            .interact()?
    } else {
        // Non-interactive: install for all detected agents
        defaults
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| if d { Some(i) } else { None })
            .collect()
    };

    if chosen.is_empty() {
        println!("No agent skills installed.");
    } else {
        println!();
        for i in &chosen {
            let target = &TARGETS[*i];
            let skill_dir = path.join(target.skill_path);
            fs::create_dir_all(&skill_dir)?;
            fs::write(skill_dir.join("SKILL.md"), SKILL_MD)?;
            println!(
                "  {} → {}/SKILL.md",
                style(target.name).cyan(),
                target.skill_path
            );
        }
    }

    println!(
        "\n{}",
        style(format!("Done. Initialized in {}", path.display())).dim()
    );
    Ok(())
}
