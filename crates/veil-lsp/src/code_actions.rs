use std::collections::HashMap;
use std::path::Path;

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, NumberOrString, Position,
    TextEdit, Url, WorkspaceEdit,
};
use veil_core::DEFAULT_PLACEHOLDER;

use crate::document_store::{byte_range_for_lsp_range, line_byte_bounds};

pub fn code_actions(
    uri: &Url,
    language_id: &str,
    text: &str,
    diagnostics: &[Diagnostic],
) -> Vec<CodeActionOrCommand> {
    diagnostics
        .iter()
        .flat_map(|diagnostic| {
            [
                mask_code_action(uri, text, diagnostic),
                inline_ignore_code_action(uri, language_id, text, diagnostic),
            ]
        })
        .flatten()
        .map(CodeActionOrCommand::CodeAction)
        .collect()
}

fn mask_code_action(uri: &Url, text: &str, diagnostic: &Diagnostic) -> Option<CodeAction> {
    if !supports_action(diagnostic, "mask") {
        return None;
    }

    let byte_range = byte_range_for_lsp_range(text, diagnostic.range)?;
    if byte_range.is_empty() || text.get(byte_range) == Some(DEFAULT_PLACEHOLDER) {
        return None;
    }

    Some(CodeAction {
        title: "Mask value".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(workspace_edit(
            uri,
            diagnostic.range,
            DEFAULT_PLACEHOLDER.to_string(),
        )),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

fn inline_ignore_code_action(
    uri: &Url,
    language_id: &str,
    text: &str,
    diagnostic: &Diagnostic,
) -> Option<CodeAction> {
    if !supports_action(diagnostic, "ignore") {
        return None;
    }

    let rule_id = match diagnostic.code.as_ref() {
        Some(NumberOrString::String(rule_id)) => rule_id.as_str(),
        _ => return None,
    };

    let comment_style = comment_style_for_document(uri, language_id)?;
    let (line_start, line_end) = line_byte_bounds(text, diagnostic.range.start.line)?;
    let line_text = &text[line_start..line_end];
    if line_text.contains("veil:ignore") {
        return None;
    }

    let insertion = comment_style.render(rule_id);
    let range = Position {
        line: diagnostic.range.start.line,
        character: utf16_len(line_text),
    };

    Some(CodeAction {
        title: "Add inline ignore".to_string(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(workspace_edit(uri, range_to_insertion(range), insertion)),
        command: None,
        is_preferred: Some(false),
        disabled: None,
        data: None,
    })
}

fn workspace_edit(
    uri: &Url,
    range: tower_lsp::lsp_types::Range,
    new_text: String,
) -> WorkspaceEdit {
    WorkspaceEdit {
        changes: Some(HashMap::from([(
            uri.clone(),
            vec![TextEdit { range, new_text }],
        )])),
        document_changes: None,
        change_annotations: None,
    }
}

fn range_to_insertion(position: Position) -> tower_lsp::lsp_types::Range {
    tower_lsp::lsp_types::Range {
        start: position,
        end: position,
    }
}

fn supports_action(diagnostic: &Diagnostic, action_name: &str) -> bool {
    diagnostic
        .data
        .as_ref()
        .and_then(|data| data.get("actions"))
        .and_then(|actions| actions.as_array())
        .is_some_and(|actions| {
            actions
                .iter()
                .any(|action| action.as_str().is_some_and(|action| action == action_name))
        })
        && matches!(diagnostic.code.as_ref(), Some(NumberOrString::String(_)))
}

fn utf16_len(text: &str) -> u32 {
    text.chars()
        .map(|character| character.len_utf16() as u32)
        .sum()
}

fn comment_style_for_document(uri: &Url, language_id: &str) -> Option<CommentStyle> {
    comment_style_for_language_id(language_id).or_else(|| {
        Path::new(uri.path())
            .extension()
            .and_then(|extension| extension.to_str())
            .and_then(comment_style_for_extension)
    })
}

fn comment_style_for_language_id(language_id: &str) -> Option<CommentStyle> {
    match language_id {
        "rust" | "go" | "javascript" | "javascriptreact" | "typescript" | "typescriptreact"
        | "java" | "c" | "cpp" | "objective-c" | "objective-cpp" => Some(CommentStyle::Line("//")),
        "python" | "ruby" | "shellscript" | "yaml" | "toml" => Some(CommentStyle::Line("#")),
        "html" | "xml" => Some(CommentStyle::Block {
            start: "<!--",
            end: "-->",
        }),
        "sql" => Some(CommentStyle::Line("--")),
        "json" | "jsonc" => None,
        _ => None,
    }
}

fn comment_style_for_extension(extension: &str) -> Option<CommentStyle> {
    match extension {
        "rs" | "go" | "js" | "jsx" | "ts" | "tsx" | "java" | "c" | "cc" | "cpp" | "cxx" | "h"
        | "hpp" | "m" | "mm" => Some(CommentStyle::Line("//")),
        "py" | "rb" | "sh" | "bash" | "zsh" | "yml" | "yaml" | "toml" => {
            Some(CommentStyle::Line("#"))
        }
        "html" | "htm" | "xml" | "svg" => Some(CommentStyle::Block {
            start: "<!--",
            end: "-->",
        }),
        "sql" => Some(CommentStyle::Line("--")),
        "json" => None,
        _ => None,
    }
}

#[derive(Clone, Copy)]
enum CommentStyle {
    Line(&'static str),
    Block {
        start: &'static str,
        end: &'static str,
    },
}

impl CommentStyle {
    fn render(self, rule_id: &str) -> String {
        match self {
            CommentStyle::Line(prefix) => format!(" {prefix} veil:ignore={rule_id}"),
            CommentStyle::Block { start, end } => {
                format!(" {start} veil:ignore={rule_id} {end}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tower_lsp::lsp_types::{NumberOrString, Range};

    fn finding_diagnostic() -> Diagnostic {
        Diagnostic {
            range: Range {
                start: Position {
                    line: 0,
                    character: 8,
                },
                end: Position {
                    line: 0,
                    character: 24,
                },
            },
            severity: None,
            code: Some(NumberOrString::String("secret.test".to_string())),
            code_description: None,
            source: Some("veil".to_string()),
            message: "Sensitive data detected".to_string(),
            related_information: None,
            tags: None,
            data: Some(json!({
                "ruleId": "secret.test",
                "score": 92,
                "grade": "CRITICAL",
                "maskedSnippet": "token = <REDACTED>",
                "actions": ["mask", "ignore"],
            })),
        }
    }

    #[test]
    fn code_actions_return_mask_and_inline_ignore_for_rust() {
        let uri = Url::parse("file:///tmp/example.rs").expect("uri");
        let diagnostics = code_actions(
            &uri,
            "rust",
            "token = raw-secret-value",
            &[finding_diagnostic()],
        );

        assert_eq!(diagnostics.len(), 2);

        let CodeActionOrCommand::CodeAction(mask_action) = &diagnostics[0] else {
            panic!("expected code action");
        };
        assert_eq!(mask_action.title, "Mask value");
        let mask_edit = mask_action.edit.as_ref().expect("workspace edit");
        let mask_changes = mask_edit.changes.as_ref().expect("text edits");
        let mask_text_edits = mask_changes.get(&uri).expect("uri edits");
        assert_eq!(mask_text_edits[0].range, finding_diagnostic().range);
        assert_eq!(mask_text_edits[0].new_text, DEFAULT_PLACEHOLDER);

        let CodeActionOrCommand::CodeAction(ignore_action) = &diagnostics[1] else {
            panic!("expected code action");
        };
        assert_eq!(ignore_action.title, "Add inline ignore");
        let ignore_edit = ignore_action.edit.as_ref().expect("workspace edit");
        let ignore_changes = ignore_edit.changes.as_ref().expect("text edits");
        let ignore_text_edits = ignore_changes.get(&uri).expect("uri edits");
        assert_eq!(
            ignore_text_edits[0].range,
            Range {
                start: Position {
                    line: 0,
                    character: 24,
                },
                end: Position {
                    line: 0,
                    character: 24,
                },
            }
        );
        assert_eq!(ignore_text_edits[0].new_text, " // veil:ignore=secret.test");
    }

    #[test]
    fn code_actions_hide_inline_ignore_for_json() {
        let uri = Url::parse("file:///tmp/example.json").expect("uri");
        let diagnostics = code_actions(
            &uri,
            "json",
            "token = raw-secret-value",
            &[finding_diagnostic()],
        );

        assert_eq!(diagnostics.len(), 1);
        let CodeActionOrCommand::CodeAction(action) = &diagnostics[0] else {
            panic!("expected code action");
        };
        assert_eq!(action.title, "Mask value");
    }

    #[test]
    fn code_actions_hide_inline_ignore_when_line_already_contains_ignore() {
        let uri = Url::parse("file:///tmp/example.rs").expect("uri");
        let diagnostics = code_actions(
            &uri,
            "rust",
            "token = raw-secret-value // veil:ignore=secret.test",
            &[finding_diagnostic()],
        );

        assert_eq!(diagnostics.len(), 1);
        let CodeActionOrCommand::CodeAction(action) = &diagnostics[0] else {
            panic!("expected code action");
        };
        assert_eq!(action.title, "Mask value");
    }

    #[test]
    fn code_actions_ignore_skip_diagnostics() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let diagnostic = Diagnostic {
            range: Range::default(),
            severity: None,
            code: Some(NumberOrString::String("MAX_FILE_SIZE".to_string())),
            code_description: None,
            source: Some("veil".to_string()),
            message: "Scan skipped".to_string(),
            related_information: None,
            tags: None,
            data: Some(json!({
                "ruleId": "MAX_FILE_SIZE",
                "actions": [],
            })),
        };

        assert!(code_actions(&uri, "text", "oversized", &[diagnostic]).is_empty());
    }

    #[test]
    fn code_actions_ignore_already_redacted_ranges() {
        let uri = Url::parse("file:///tmp/example.rs").expect("uri");
        let diagnostics = code_actions(&uri, "rust", "token = <REDACTED>", &[finding_diagnostic()]);

        assert_eq!(diagnostics.len(), 1);
        let CodeActionOrCommand::CodeAction(action) = &diagnostics[0] else {
            panic!("expected code action");
        };
        assert_eq!(action.title, "Add inline ignore");
    }
}
