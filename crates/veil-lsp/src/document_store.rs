use std::collections::HashMap;
use std::fmt;
use std::ops::Range;

use tower_lsp::lsp_types::{
    Position as LspPosition, Range as LspRange, TextDocumentContentChangeEvent, Url,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentState {
    pub uri: Url,
    pub language_id: String,
    pub text: String,
    pub version: i32,
    pub scan_revision: u64,
}

#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<Url, DocumentState>,
}

impl DocumentStore {
    pub fn open(
        &mut self,
        uri: Url,
        language_id: String,
        text: String,
        version: i32,
    ) -> DocumentState {
        let document = DocumentState {
            uri: uri.clone(),
            language_id,
            text,
            version,
            scan_revision: 0,
        };
        self.documents.insert(uri, document.clone());
        document
    }

    pub fn apply_changes(
        &mut self,
        uri: &Url,
        version: i32,
        content_changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Result<Option<DocumentState>, DocumentChangeError> {
        let Some(document) = self.documents.get_mut(uri) else {
            return Ok(None);
        };

        for change in content_changes {
            apply_content_change(&mut document.text, change)?;
        }
        document.version = version;
        document.scan_revision = document.scan_revision.saturating_add(1);

        Ok(Some(document.clone()))
    }

    pub fn get(&self, uri: &Url) -> Option<DocumentState> {
        self.documents.get(uri).cloned()
    }

    pub fn has_revision(&self, uri: &Url, scan_revision: u64) -> bool {
        self.documents
            .get(uri)
            .is_some_and(|document| document.scan_revision == scan_revision)
    }

    pub fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentChangeError {
    InvalidRange { range: LspRange },
}

impl fmt::Display for DocumentChangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentChangeError::InvalidRange { range } => {
                write!(formatter, "invalid text document change range: {range:?}")
            }
        }
    }
}

impl std::error::Error for DocumentChangeError {}

fn apply_content_change(
    text: &mut String,
    change: TextDocumentContentChangeEvent,
) -> Result<(), DocumentChangeError> {
    let Some(range) = change.range else {
        *text = change.text;
        return Ok(());
    };

    let Some(byte_range) = byte_range_for_lsp_range(text, range) else {
        return Err(DocumentChangeError::InvalidRange { range });
    };

    text.replace_range(byte_range, &change.text);
    Ok(())
}

pub fn byte_range_for_lsp_range(text: &str, range: LspRange) -> Option<Range<usize>> {
    let start = byte_index_for_position(text, range.start)?;
    let end = byte_index_for_position(text, range.end)?;
    (start <= end).then_some(start..end)
}

pub fn line_byte_bounds(text: &str, target_line: u32) -> Option<(usize, usize)> {
    let mut line = 0_u32;
    let mut line_start = 0_usize;

    for (byte_offset, character) in text.char_indices() {
        if character != '\n' {
            continue;
        }

        if line == target_line {
            let line_end = text[line_start..byte_offset]
                .strip_suffix('\r')
                .map_or(byte_offset, |line_without_cr| {
                    line_start + line_without_cr.len()
                });
            return Some((line_start, line_end));
        }

        line = line.saturating_add(1);
        line_start = byte_offset + character.len_utf8();
    }

    (line == target_line).then_some((line_start, text.len()))
}

fn byte_index_for_position(text: &str, position: LspPosition) -> Option<usize> {
    let (line_start, line_end) = line_byte_bounds(text, position.line)?;
    let line = &text[line_start..line_end];

    if position.character == 0 {
        return Some(line_start);
    }

    let mut utf16_units = 0_u32;
    for (byte_offset, character) in line.char_indices() {
        if utf16_units == position.character {
            return Some(line_start + byte_offset);
        }

        utf16_units = utf16_units.saturating_add(character.len_utf16() as u32);
        if utf16_units > position.character {
            return None;
        }
    }

    (utf16_units == position.character).then_some(line_end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::Position;

    fn full_change(text: &str) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_string(),
        }
    }

    fn ranged_change(range: LspRange, text: &str) -> TextDocumentContentChangeEvent {
        TextDocumentContentChangeEvent {
            range: Some(range),
            range_length: None,
            text: text.to_string(),
        }
    }

    #[test]
    fn store_applies_full_document_changes() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let mut store = DocumentStore::default();
        store.open(uri.clone(), "text".to_string(), "before".to_string(), 1);

        let document = store
            .apply_changes(&uri, 2, vec![full_change("after")])
            .expect("change")
            .expect("document");

        assert_eq!(document.text, "after");
        assert_eq!(document.version, 2);
        assert_eq!(document.scan_revision, 1);
    }

    #[test]
    fn store_applies_incremental_changes_with_utf16_positions() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let mut store = DocumentStore::default();
        store.open(uri.clone(), "text".to_string(), "a🙂c".to_string(), 1);

        let document = store
            .apply_changes(
                &uri,
                2,
                vec![ranged_change(
                    LspRange {
                        start: Position {
                            line: 0,
                            character: 1,
                        },
                        end: Position {
                            line: 0,
                            character: 3,
                        },
                    },
                    "b",
                )],
            )
            .expect("change")
            .expect("document");

        assert_eq!(document.text, "abc");
        assert_eq!(document.scan_revision, 1);
    }

    #[test]
    fn store_rejects_positions_inside_utf16_surrogate_pairs() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let mut store = DocumentStore::default();
        store.open(uri.clone(), "text".to_string(), "a🙂c".to_string(), 1);

        let result = store.apply_changes(
            &uri,
            2,
            vec![ranged_change(
                LspRange {
                    start: Position {
                        line: 0,
                        character: 2,
                    },
                    end: Position {
                        line: 0,
                        character: 3,
                    },
                },
                "b",
            )],
        );

        assert!(matches!(
            result,
            Err(DocumentChangeError::InvalidRange { .. })
        ));
    }

    #[test]
    fn open_resets_scan_revision_for_new_document_state() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let mut store = DocumentStore::default();

        let original = store.open(uri.clone(), "rust".to_string(), "before".to_string(), 1);
        assert_eq!(original.scan_revision, 0);
        assert_eq!(original.language_id, "rust");

        let updated = store
            .apply_changes(&uri, 2, vec![full_change("after")])
            .expect("change")
            .expect("document");
        assert_eq!(updated.scan_revision, 1);

        let reopened = store.open(uri, "rust".to_string(), "reopened".to_string(), 1);
        assert_eq!(reopened.scan_revision, 0);
    }

    #[test]
    fn has_revision_tracks_latest_document_generation() {
        let uri = Url::parse("file:///tmp/example.txt").expect("uri");
        let mut store = DocumentStore::default();
        store.open(uri.clone(), "text".to_string(), "before".to_string(), 1);

        let updated = store
            .apply_changes(&uri, 2, vec![full_change("after")])
            .expect("change")
            .expect("document");

        assert!(store.has_revision(&uri, updated.scan_revision));
        assert!(!store.has_revision(&uri, 0));
    }
}
