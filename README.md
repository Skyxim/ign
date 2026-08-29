# ign 🦀

[![Crates.io](https://img.shields.io/crates/v/ign.svg)](https://crates.io/crates/ign)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/Skyxim/ign/actions/workflows/ci.yml/badge.svg)](https://github.com/Skyxim/ign/actions/workflows/ci.yml)

A fast, flexible, and feature-rich `.gitignore` CLI manager written in Rust, powered by the official [github/gitignore](https://github.com/github/gitignore) repository.

---

## Features

- ⚡ **Lightning Fast & Offline-First**: Zero implicit network requests on startup; manages templates locally with on-demand synchronization.
- 📦 **Modular Block Isolation**: Automatically generates clear `BEGIN`/`END` delimiters for each applied template.
- 🛡️ **Guaranteed Custom Rule Preservation**: Manually added lines and custom user rules are safely protected in a `# --- BEGIN CUSTOM RULES ---` block.
- 🔍 **Fuzzy Search & Smart Aliases**: Search templates with typo tolerance or use intuitive aliases like `macos`, `vscode`, `idea`, `rust`, `python`.
- 📊 **Status & Metadata Inspection**: Read applied templates, creation time, and custom rules in human-readable or structured `--json` format.
- 🤖 **Agent & CI Ready**: Includes a universal Agent Skill and JSON output for seamless automated workflows.

---

## Installation

### Via Cargo
```bash
cargo install ign
```

### From Source
```bash
git clone https://github.com/Skyxim/ign.git
cd gign
cargo install --path .
```

### Install the Agent Skill
Install the `gitignore` Agent Skill with `npx skills`:
```bash
npx skills add Skyxim/ign --skill gitignore
```

---

## Quick Start

### 1. Initialize the Template Library
```bash
ign template init
```

### 2. Generate `.gitignore` for Your Project
```bash
# Combine multiple templates
ign init Rust macOS Linux Visual_Studio_Code JetBrains

# Or interactive multi-select menu
ign init -i
```

### 3. Safely Append or Remove Templates
```bash
# Append a template (auto-deduplicated, preserves custom rules)
ign add Python

# Remove a template
ign remove macos
```

### 4. Inspect Status
```bash
# Terminal overview
ign status

# Structured JSON for scripts / Agent
ign status --json
```

### 5. Search & Explore Templates
```bash
ign search flutter
ign list --category global
ign show Rust
```

### 6. Synchronize Latest Templates from GitHub
```bash
ign template update
# Or top-level shorthand:
ign update
```

---

## License

This project is licensed under the [MIT License](LICENSE).
