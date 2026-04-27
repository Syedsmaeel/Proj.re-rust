//! Maps detected foreign tools to their Rust equivalents

use crate::{tools::known_translations, ForeignTool, Translation};

/// Translate a single foreign tool to its Rust equivalent
pub fn translate(tool: &ForeignTool) -> Translation {
    let map = known_translations();
    let equivalent = map.get(tool.name.as_str()).cloned();

    Translation {
        foreign: tool.clone(),
        equivalent,
    }
}

/// Translate all detected tools
pub fn translate_all(tools: &[ForeignTool]) -> Vec<Translation> {
    tools.iter().map(translate).collect()
}

/// Return only tools that have a known Rust equivalent
pub fn known_only(translations: Vec<Translation>) -> Vec<Translation> {
    translations.into_iter().filter(|t| t.equivalent.is_some()).collect()
}

/// Return tools with no known Rust equivalent yet
pub fn unknown_only(translations: Vec<Translation>) -> Vec<Translation> {
    translations.into_iter().filter(|t| t.equivalent.is_none()).collect()
}
