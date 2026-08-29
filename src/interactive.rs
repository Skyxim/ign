use anyhow::{anyhow, Result};
use inquire::MultiSelect;

use crate::models::TemplateInfo;

/// Interactively prompt the user to select templates using fuzzy search and multi-select
pub fn select_templates_interactively(templates: &[TemplateInfo]) -> Result<Vec<String>> {
    if templates.is_empty() {
        return Err(anyhow!("No templates available to select from"));
    }

    let items: Vec<String> = templates
        .iter()
        .map(|t| format!("[{}] {}", t.category, t.name))
        .collect();

    let ans = MultiSelect::new("Select .gitignore templates to apply:", items)
        .with_page_size(18)
        .with_help_message("↑↓ to navigate, Space to select/unselect, Type to filter, Enter to confirm")
        .prompt();

    match ans {
        Ok(selected_items) => {
            let names = selected_items
                .into_iter()
                .filter_map(|item| {
                    // Extract template name from "[Category] Name"
                    if let Some(pos) = item.find(']') {
                        Some(item[pos + 1..].trim().to_string())
                    } else {
                        Some(item)
                    }
                })
                .collect();
            Ok(names)
        }
        Err(e) => Err(anyhow!("Interactive selection cancelled or failed: {}", e)),
    }
}
