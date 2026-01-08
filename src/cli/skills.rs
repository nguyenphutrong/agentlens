use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Supported AI coding tools that use agent skills
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SkillTarget {
    Claude,   // ~/.claude/skills/
    OpenCode, // ~/.config/opencode/skill/
    Codex,    // ~/.codex/skills/
}

impl std::fmt::Display for SkillTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillTarget::Claude => write!(f, "Claude Code"),
            SkillTarget::OpenCode => write!(f, "OpenCode"),
            SkillTarget::Codex => write!(f, "Codex CLI"),
        }
    }
}

impl SkillTarget {
    /// Get the skill directory path for this target
    pub fn skill_dir(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        match self {
            SkillTarget::Claude => home.join(".claude").join("skills").join("agentlens"),
            SkillTarget::OpenCode => home
                .join(".config")
                .join("opencode")
                .join("skill")
                .join("agentlens"),
            SkillTarget::Codex => home.join(".codex").join("skills").join("agentlens"),
        }
    }

    /// Check if this tool appears to be installed
    pub fn is_installed(&self) -> bool {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        match self {
            SkillTarget::Claude => home.join(".claude").exists(),
            SkillTarget::OpenCode => home.join(".config").join("opencode").exists(),
            SkillTarget::Codex => home.join(".codex").exists(),
        }
    }

    /// Check if agentlens skill is already installed for this target
    pub fn skill_installed(&self) -> bool {
        self.skill_dir().join("SKILL.md").exists()
    }
}

// ============================================================================
// SKILL CONTENT - Embedded skill files
// ============================================================================

const SKILL_MD: &str = r#"---
name: agentlens
description: Navigate and understand codebases using agentlens hierarchical documentation. Use when exploring new projects, finding modules, locating symbols in large files, finding TODOs/warnings, or understanding code structure.
metadata:
  short-description: Codebase navigation with agentlens
  author: agentlens
  version: "1.0"
---

# AgentLens - Codebase Navigation

## Before Working on Any Codebase
Always start by reading `.agentlens/INDEX.md` for the project map.

## Navigation Hierarchy

| Level | File | Purpose |
|-------|------|---------|
| L0 | `INDEX.md` | Project overview, all modules listed |
| L1 | `modules/{slug}/MODULE.md` | Module details, file list |
| L1 | `modules/{slug}/outline.md` | Symbols in large files |
| L1 | `modules/{slug}/memory.md` | TODOs, warnings, business rules |
| L1 | `modules/{slug}/imports.md` | File dependencies |
| L2 | `files/{slug}.md` | Deep docs for complex files |

## Navigation Flow

```
INDEX.md → Find module → MODULE.md → outline.md/memory.md → Source file
```

## When To Read What

| You Need | Read This |
|----------|-----------|
| Project overview | `.agentlens/INDEX.md` |
| Find a module | INDEX.md, search module name |
| Understand a module | `modules/{slug}/MODULE.md` |
| Find function/class in large file | `modules/{slug}/outline.md` |
| Find TODOs, warnings, rules | `modules/{slug}/memory.md` |
| Understand file dependencies | `modules/{slug}/imports.md` |

## Best Practices

1. **Don't read source files directly** for large codebases - use outline.md first
2. **Check memory.md before modifying** code to see warnings and TODOs
3. **Use outline.md to locate symbols**, then read only the needed source sections
4. **Regenerate docs** with `agentlens` command if they seem stale

For detailed navigation patterns, see [references/navigation.md](references/navigation.md)
For structure explanation, see [references/structure.md](references/structure.md)
"#;

const NAVIGATION_MD: &str = r#"# Navigation Patterns

## Pattern 1: Exploring a New Codebase

```
1. Read .agentlens/INDEX.md
   → Get list of all modules
   → Note entry points and hub modules

2. Pick relevant module from INDEX
   → Read modules/{slug}/MODULE.md
   → Understand module purpose and files

3. Need specific symbol?
   → Read modules/{slug}/outline.md
   → Find line number of function/class

4. Check for issues first?
   → Read modules/{slug}/memory.md
   → See TODOs, warnings before editing
```

## Pattern 2: Finding Where Something Is Defined

```
1. Start with INDEX.md
2. Search for keyword in module descriptions
3. Go to matching MODULE.md
4. Check outline.md for symbol locations
5. Read only the specific source lines needed
```

## Pattern 3: Understanding Dependencies

```
1. Read modules/{slug}/imports.md
2. See which files import what
3. Understand the dependency graph
4. Navigate to related modules as needed
```

## Pattern 4: Before Modifying Code

```
1. Read memory.md for the module
2. Check for:
   - TODO: Pending work
   - FIXME: Known bugs
   - WARNING: Dangerous areas
   - SAFETY: Critical invariants
   - DEPRECATED: Code to avoid
3. Understand the context before changes
```

## Token Efficiency Tips

- **Never read entire source files** in large codebases
- **Use outline.md** to find exact line numbers first
- **Read only relevant sections** of source code
- **Navigate hierarchically**: INDEX → MODULE → outline → source
- **Estimated savings**: 80-96% fewer tokens than reading raw source
"#;

const STRUCTURE_MD: &str = r#"# AgentLens Output Structure

## Directory Layout

```
.agentlens/
├── INDEX.md              # L0: Global routing table
├── AGENT.md              # Agent-specific instructions
├── modules/
│   └── {module-slug}/
│       ├── MODULE.md     # L1: Module overview
│       ├── outline.md    # L1: Symbol maps for large files
│       ├── memory.md     # L1: TODOs, warnings, rules
│       └── imports.md    # L1: File dependencies
└── files/
    └── {file-slug}.md    # L2: Deep docs for complex files
```

## File Purposes

### INDEX.md (Always Read First)
- Project name and description
- Complete list of modules with descriptions
- Entry points (main files)
- Hub files (heavily imported)
- High-priority warnings summary

### MODULE.md
- Module purpose and responsibility
- List of all files in the module
- File descriptions and line counts
- Language breakdown

### outline.md
- Symbol maps for large files (>500 lines)
- Functions, classes, structs, enums, traits
- Line numbers for quick navigation
- Visibility (public/private)

### memory.md
- TODO comments
- FIXME and BUG markers
- WARNING and SAFETY notes
- DEPRECATED markers
- Business rules (RULE, POLICY)

### imports.md
- Which files import which
- Internal dependencies within module
- Helps understand coupling

### files/{slug}.md (L2 - Complex Files Only)
- Generated for very complex files
- Detailed symbol documentation
- More context than outline.md
"#;

// ============================================================================
// DETECTION & INSTALLATION
// ============================================================================

/// Detect which AI tool is most likely being used
pub fn detect_skill_target() -> Option<SkillTarget> {
    // Priority: Claude > OpenCode > Codex
    if SkillTarget::Claude.is_installed() {
        return Some(SkillTarget::Claude);
    }
    if SkillTarget::OpenCode.is_installed() {
        return Some(SkillTarget::OpenCode);
    }
    if SkillTarget::Codex.is_installed() {
        return Some(SkillTarget::Codex);
    }
    None
}

/// Install skills to specified target(s)
pub fn install_skills(claude: bool, opencode: bool, codex: bool, all: bool) -> Result<()> {
    let targets = if all {
        vec![
            SkillTarget::Claude,
            SkillTarget::OpenCode,
            SkillTarget::Codex,
        ]
    } else if claude || opencode || codex {
        let mut t = Vec::new();
        if claude {
            t.push(SkillTarget::Claude);
        }
        if opencode {
            t.push(SkillTarget::OpenCode);
        }
        if codex {
            t.push(SkillTarget::Codex);
        }
        t
    } else {
        // Auto-detect
        match detect_skill_target() {
            Some(target) => vec![target],
            None => {
                eprintln!("No supported AI tool detected.");
                eprintln!("Supported tools: Claude Code, OpenCode, Codex CLI");
                eprintln!();
                eprintln!("Use explicit flags to install:");
                eprintln!("  agentlens skills install --claude");
                eprintln!("  agentlens skills install --opencode");
                eprintln!("  agentlens skills install --codex");
                eprintln!("  agentlens skills install --all");
                return Ok(());
            }
        }
    };

    for target in targets {
        install_skill_to_target(target)?;
    }

    eprintln!();
    eprintln!("Skill installed! The AI agent can now use agentlens for codebase navigation.");
    eprintln!("Restart your AI tool to load the new skill.");

    Ok(())
}

fn install_skill_to_target(target: SkillTarget) -> Result<()> {
    let skill_dir = target.skill_dir();
    let references_dir = skill_dir.join("references");

    eprintln!("Installing agentlens skill for {}...", target);

    // Create directories
    fs::create_dir_all(&references_dir).context(format!(
        "Failed to create skill directory: {}",
        skill_dir.display()
    ))?;

    // Write SKILL.md
    let skill_path = skill_dir.join("SKILL.md");
    if skill_path.exists() {
        eprintln!("  SKILL.md already exists, updating...");
    }
    fs::write(&skill_path, SKILL_MD).context("Failed to write SKILL.md")?;
    eprintln!("  Created: {}", skill_path.display());

    // Write references/navigation.md
    let nav_path = references_dir.join("navigation.md");
    fs::write(&nav_path, NAVIGATION_MD).context("Failed to write navigation.md")?;
    eprintln!("  Created: {}", nav_path.display());

    // Write references/structure.md
    let struct_path = references_dir.join("structure.md");
    fs::write(&struct_path, STRUCTURE_MD).context("Failed to write structure.md")?;
    eprintln!("  Created: {}", struct_path.display());

    Ok(())
}

/// Remove skills from all known locations
pub fn remove_skills() -> Result<()> {
    let targets = [
        SkillTarget::Claude,
        SkillTarget::OpenCode,
        SkillTarget::Codex,
    ];
    let mut removed_any = false;

    for target in targets {
        let skill_dir = target.skill_dir();
        if skill_dir.exists() {
            eprintln!("Removing agentlens skill from {}...", target);
            fs::remove_dir_all(&skill_dir)
                .context(format!("Failed to remove {}", skill_dir.display()))?;
            eprintln!("  Removed: {}", skill_dir.display());
            removed_any = true;
        }
    }

    if removed_any {
        eprintln!();
        eprintln!("Agentlens skills removed.");
    } else {
        eprintln!("No agentlens skills found to remove.");
    }

    Ok(())
}

/// List installed skills and their locations
pub fn list_skills() -> Result<()> {
    let targets = [
        SkillTarget::Claude,
        SkillTarget::OpenCode,
        SkillTarget::Codex,
    ];

    eprintln!("AgentLens Skill Status:");
    eprintln!();

    let mut found_any = false;

    for target in targets {
        let installed = target.skill_installed();
        let tool_exists = target.is_installed();
        let skill_dir = target.skill_dir();

        let status = if installed {
            found_any = true;
            "✓ Installed"
        } else if tool_exists {
            "○ Not installed (tool detected)"
        } else {
            "- Not installed (tool not found)"
        };

        eprintln!("  {:<12} {} ", format!("{}:", target), status);
        if installed {
            eprintln!("               {}", skill_dir.display());
        }
    }

    if !found_any {
        eprintln!();
        eprintln!("No agentlens skills installed. Run:");
        eprintln!("  agentlens skills install");
    }

    Ok(())
}

// ============================================================================
// PROJECT-LEVEL SKILLS (Optional)
// ============================================================================

#[allow(dead_code)]
pub fn install_skill_to_project(path: &Path, target: SkillTarget) -> Result<()> {
    let skill_dir = match target {
        SkillTarget::Claude => path.join(".claude").join("skills").join("agentlens"),
        SkillTarget::OpenCode => path.join(".opencode").join("skill").join("agentlens"),
        SkillTarget::Codex => path.join(".codex").join("skills").join("agentlens"),
    };

    let references_dir = skill_dir.join("references");

    eprintln!("Installing project-level agentlens skill for {}...", target);

    fs::create_dir_all(&references_dir).context(format!(
        "Failed to create skill directory: {}",
        skill_dir.display()
    ))?;

    fs::write(skill_dir.join("SKILL.md"), SKILL_MD)?;
    fs::write(references_dir.join("navigation.md"), NAVIGATION_MD)?;
    fs::write(references_dir.join("structure.md"), STRUCTURE_MD)?;

    eprintln!("  Created: {}", skill_dir.display());

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_skill_target_display() {
        assert_eq!(format!("{}", SkillTarget::Claude), "Claude Code");
        assert_eq!(format!("{}", SkillTarget::OpenCode), "OpenCode");
        assert_eq!(format!("{}", SkillTarget::Codex), "Codex CLI");
    }

    #[test]
    fn test_install_skill_to_project() {
        let temp = TempDir::new().unwrap();

        install_skill_to_project(temp.path(), SkillTarget::Claude).unwrap();

        let skill_dir = temp.path().join(".claude").join("skills").join("agentlens");
        assert!(skill_dir.join("SKILL.md").exists());
        assert!(skill_dir.join("references").join("navigation.md").exists());
        assert!(skill_dir.join("references").join("structure.md").exists());
    }

    #[test]
    fn test_skill_content_valid() {
        // Verify SKILL.md has required frontmatter
        assert!(SKILL_MD.contains("name: agentlens"));
        assert!(SKILL_MD.contains("description:"));

        // Verify content is meaningful
        assert!(SKILL_MD.contains("INDEX.md"));
        assert!(SKILL_MD.contains("MODULE.md"));
        assert!(NAVIGATION_MD.contains("Pattern"));
        assert!(STRUCTURE_MD.contains(".agentlens/"));
    }
}
