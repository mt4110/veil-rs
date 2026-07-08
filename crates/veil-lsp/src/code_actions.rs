use std::collections::HashMap;

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Diagnostic, NumberOrString, TextEdit, Url,
    WorkspaceEdit,
};
use veil_core::DEFAULT_PLACEHOLDER;

use crate::document_store::byte_range_for_lsp_range;

pub fn mask_code_actions(
    uri: &Url,
    text: &str,
    diagnostics: &[Diagnostic],
) -> Vec<CodeActionOrCommand> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| mask_code_action(uri, text, diagnostic))
        .map(CodeActionOrCommand::CodeAction)
        .collect()
}

fn mask_code_action(uri: &Url, text: &str, diagnostic: &Diagnostic) -> Option<CodeAction> {
    if !supports_mask_action(diagnostic) {
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
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(
                uri.clone(),
                vec![TextEdit {
                    range: diagnostic.range,
                    new_text: DEFAULT_PLACEHOLDER.to_string(),
                }],
            )])),
            document_changes: None,
            change_annotations: None,
        }),
        command: None,
        is_preferred: Some(true),
        disabled: None,
        data: None,
    })
}

fn supports_mask_action(diagnostic: &Diagnostic) -> bool {
    diagnostic
        .data
        .as_ref()
        .and_then(|data| data.get("actions"))
        .and_then(|actions| actions.as_array())
        .is_some_and(|actions| {
            actions
                .iter()
                .any(|action| action.as_str().is_some_and(|action| action == "mask"))
        })
        && matches!(diagnostic.code.as_ref(), Some(NumberOrString::String(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tower_lsp::lsp_types::{NumberOrString, Position, Range};

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
    fn mask_code_actions_return_quickfix_edit() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let diagnostics =
            mask_code_actions(&uri, "token = raw-secret-value", &[finding_diagnostic()]);

        assert_eq!(diagnostics.len(), 1);
        let CodeActionOrCommand::CodeAction(action) = &diagnostics[0] else {
            panic!("expected code action");
        };

        assert_eq!(action.title, "Mask value");
        assert_eq!(action.kind, Some(CodeActionKind::QUICKFIX));
        let edit = action.edit.as_ref().expect("workspace edit");
        let changes = edit.changes.as_ref().expect("text edits");
        let text_edits = changes.get(&uri).expect("uri edits");
        assert_eq!(text_edits.len(), 1);
        assert_eq!(text_edits[0].range, finding_diagnostic().range);
        assert_eq!(text_edits[0].new_text, DEFAULT_PLACEHOLDER);
    }

    #[test]
    fn mask_code_actions_ignore_skip_diagnostics() {
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

        assert!(mask_code_actions(&uri, "oversized", &[diagnostic]).is_empty());
    }

    #[test]
    fn mask_code_actions_ignore_already_redacted_ranges() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");

        assert!(mask_code_actions(&uri, "token = <REDACTED>", &[finding_diagnostic()]).is_empty());
    }
}
