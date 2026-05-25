use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::{DocumentSymbol, SymbolKind, Range, Position};

#[derive(Debug, Clone)]
pub struct ParsedFountain {
    pub tokens: Vec<betterfountain_rust::ScriptToken>,
    pub properties: betterfountain_rust::ScreenplayProperties,
    pub statistics: Option<betterfountain_rust::statistics::Statistics>,
}

impl ParsedFountain {
    pub fn to_document_symbols(&self) -> Vec<DocumentSymbol> {
        let title = self.properties.title_keys.first()
            .cloned()
            .unwrap_or_else(|| "Untitled Script".to_string());

        let mut root_children: Vec<DocumentSymbol> = Vec::new();
        let mut notes_items: Vec<DocumentSymbol> = Vec::new();
        let mut bookmarks_items: Vec<DocumentSymbol> = Vec::new();

        // 遍历顶层结构
        for token in &self.properties.structure {
            // 顶层 isnote 直接收集到 notes_items
            if token.isnote {
                notes_items.push(self.create_note_symbol(token));
                continue;
            }
            // 顶层 is_bookmark 直接收集到 bookmarks_items
            if token.is_bookmark {
                bookmarks_items.push(self.create_bookmark_symbol(token));
                continue;
            }
            // 其他 token 正常构建 symbol
            let mut symbol = self.token_to_document_symbol(token);
            let mut children: Vec<DocumentSymbol> = Vec::new();
            
            // 递��处理子节点（子节点中的 note/bookmark 保留在原位置）
            self.build_children_tree(&token.children, &mut children, &mut notes_items, &mut bookmarks_items);
            self.build_children_tree(&token.structs, &mut children, &mut notes_items, &mut bookmarks_items);
            
            // 添加 synopses 作为当前节点的 children
            for syn in &token.synopses {
                let range = Range::new(
                    Position::new(syn.line as u32, 0),
                    Position::new(syn.line as u32, syn.synopsis.len() as u32)
                );
                children.push(DocumentSymbol {
                    name: syn.synopsis.clone(),
                    detail: None,
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    deprecated: None,
                    range: range.clone(),
                    selection_range: range,
                    children: None,
                });
            }
            
            // 添加 token.notes（附属注解）作为当前节点的 children
            for note in &token.notes {
                let range = Range::new(
                    Position::new(note.line as u32, 0),
                    Position::new(note.line as u32, note.note.len() as u32)
                );
                children.push(DocumentSymbol {
                    name: note.note.clone(),
                    detail: None,
                    kind: SymbolKind::CONSTANT,
                    tags: None,
                    deprecated: None,
                    range: range.clone(),
                    selection_range: range,
                    children: None,
                });
            }
            
            if !children.is_empty() {
                children.sort_by_key(|s| s.selection_range.start.line);
                symbol.children = Some(children);
            }
            
            root_children.push(symbol);
        }

        root_children.sort_by_key(|s| s.selection_range.start.line);
        notes_items.sort_by_key(|s| s.selection_range.start.line);
        bookmarks_items.sort_by_key(|s| s.selection_range.start.line);

        // NOTES 根节点
        let notes_root = if !notes_items.is_empty() {
            let start = notes_items.first().map(|s| s.range.start).unwrap_or(Position::new(0, 0));
            let end = notes_items.last().map(|s| s.range.end).unwrap_or(Position::new(0, 0));
            Some(DocumentSymbol {
                name: "NOTES".to_string(),
                detail: None,
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: Range::new(start, end),
                selection_range: Range::new(start, end),
                children: Some(notes_items),
            })
        } else {
            None
        };

        // Bookmarks 根节点
        let bookmarks_root = if !bookmarks_items.is_empty() {
            let start = bookmarks_items.first().map(|s| s.range.start).unwrap_or(Position::new(0, 0));
            let end = bookmarks_items.last().map(|s| s.range.end).unwrap_or(Position::new(0, 0));
            Some(DocumentSymbol {
                name: "Bookmarks".to_string(),
                detail: None,
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: Range::new(start, end),
                selection_range: Range::new(start, end),
                children: Some(bookmarks_items),
            })
        } else {
            None
        };

        if let Some(notes) = notes_root {
            root_children.push(notes);
        }
        if let Some(bookmarks) = bookmarks_root {
            root_children.push(bookmarks);
        }

        let root = DocumentSymbol {
            name: title,
            detail: None,
            kind: SymbolKind::FILE,
            tags: None,
            deprecated: None,
            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            selection_range: Range::new(Position::new(0, 0), Position::new(0, 0)),
            children: if root_children.is_empty() { None } else { Some(root_children) },
        };

        vec![root]
    }

    // 递归构建子节点树（子节点中的 note/bookmark 保留在原位置，不提取到顶层）
    fn build_children_tree(
        &self,
        tokens: &[betterfountain_rust::StructToken],
        parent_children: &mut Vec<DocumentSymbol>,
        _notes_items: &mut Vec<DocumentSymbol>,
        _bookmarks_items: &mut Vec<DocumentSymbol>,
    ) {
        for token in tokens {
            // 子节点如果是 note/bookmark，保留在原位置作为 children
            let mut symbol = self.token_to_document_symbol(token);
            let mut children: Vec<DocumentSymbol> = Vec::new();

            // 递归处理孙节点
            self.build_children_tree(&token.children, &mut children, _notes_items, _bookmarks_items);
            self.build_children_tree(&token.structs, &mut children, _notes_items, _bookmarks_items);

            // 添�� synopses
            for syn in &token.synopses {
                let range = Range::new(
                    Position::new(syn.line as u32, 0),
                    Position::new(syn.line as u32, syn.synopsis.len() as u32)
                );
                children.push(DocumentSymbol {
                    name: syn.synopsis.clone(),
                    detail: None,
                    kind: SymbolKind::INTERFACE,
                    tags: None,
                    deprecated: None,
                    range: range.clone(),
                    selection_range: range,
                    children: None,
                });
            }

            // 添加 token.notes（附属注解）作为当前节点的 children
            for note in &token.notes {
                let range = Range::new(
                    Position::new(note.line as u32, 0),
                    Position::new(note.line as u32, note.note.len() as u32)
                );
                children.push(DocumentSymbol {
                    name: note.note.clone(),
                    detail: None,
                    kind: SymbolKind::CONSTANT,
                    tags: None,
                    deprecated: None,
                    range: range.clone(),
                    selection_range: range,
                    children: None,
                });
            }

            if !children.is_empty() {
                children.sort_by_key(|s| s.selection_range.start.line);
                symbol.children = Some(children);
            }

            parent_children.push(symbol);
        }
    }

    fn create_note_symbol(&self, token: &betterfountain_rust::StructToken) -> DocumentSymbol {
        let line = token.id.as_ref()
            .and_then(|id| id.strip_prefix('/'))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let range = Range::new(
            Position::new(line, 0),
            Position::new(line, token.text.len() as u32)
        );

        DocumentSymbol {
            name: if token.text.is_empty() { format!("Line {}", line) } else { token.text.clone() },
            detail: None,
            kind: SymbolKind::CONSTANT,
            tags: None,
            deprecated: None,
            range: range.clone(),
            selection_range: range,
            children: None,
        }
    }

    fn create_bookmark_symbol(&self, token: &betterfountain_rust::StructToken) -> DocumentSymbol {
        let line = token.id.as_ref()
            .and_then(|id| id.strip_prefix('/'))
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);

        let range = Range::new(
            Position::new(line, 0),
            Position::new(line, token.text.len() as u32)
        );

        DocumentSymbol {
            name: if token.text.is_empty() { format!("Line {}", line) } else { token.text.clone() },
            detail: None,
            kind: SymbolKind::CONSTANT,
            tags: None,
            deprecated: None,
            range: range.clone(),
            selection_range: range,
            children: None,
        }
    }

    fn token_to_document_symbol(&self, token: &betterfountain_rust::StructToken) -> DocumentSymbol {
        let name = if token.text.is_empty() {
            if token.isscene {
                "Scene".to_string()
            } else if token.ischartor {
                "Dialogue".to_string()
            } else if token.section {
                "Section".to_string()
            } else {
                "Item".to_string()
            }
        } else {
            token.text.clone()
        };

        let kind = if token.isscene {
            SymbolKind::CLASS
        } else if token.ischartor {
            SymbolKind::VARIABLE
        } else if token.section {
            SymbolKind::NAMESPACE
        } else {
            SymbolKind::PROPERTY
        };

        let detail = if token.section {
            self.format_duration(self.calculate_section_duration(token))
        } else {
            self.format_duration(token.duration_sec)
        };

        let range = token.range.as_ref().map(|r| {
            Range::new(
                Position::new(r.start.line as u32, r.start.character as u32),
                Position::new(r.end.line as u32, r.end.character as u32)
            )
        }).unwrap_or_else(|| {
            Range::new(
                Position::new(0, 0),
                Position::new(0, 0)
            )
        });

        let selection_range = range.clone();

        let mut children: Vec<DocumentSymbol> = Vec::new();

        for child in &token.children {
            if !child.isnote && !child.is_bookmark {
                children.push(self.token_to_document_symbol(child));
            }
        }
        for child in &token.structs {
            if !child.isnote && !child.is_bookmark {
                children.push(self.token_to_document_symbol(child));
            }
        }

        let children = if children.is_empty() { None } else { Some(children) };

        DocumentSymbol {
            name,
            detail: Some(detail),
            kind,
            tags: None,
            deprecated: None,
            range,
            selection_range,
            children,
        }
    }

    fn calculate_section_duration(&self, token: &betterfountain_rust::StructToken) -> f64 {
        let mut duration = 0.0;
        for child in &token.children {
            if child.section {
                duration += self.calculate_section_duration(child);
            } else {
                duration += child.duration_sec;
            }
        }
        for child in &token.structs {
            if child.section {
                duration += self.calculate_section_duration(child);
            } else {
                duration += child.duration_sec;
            }
        }
        duration
    }

    fn format_duration(&self, seconds: f64) -> String {
        if seconds <= 0.0 {
            return String::new();
        }

        let total_seconds = seconds as u64;
        let minutes = total_seconds / 60;
        let secs = total_seconds % 60;

        if minutes > 0 {
            format!("{}m {}s", minutes, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

#[derive(Clone)]
pub struct FountainDocument {
    pub uri: String,
    pub text: String,
    pub version: i32,
    pub parsed: Option<ParsedFountain>,
}

impl FountainDocument {
    pub fn new(uri: String, text: String, version: i32) -> Self {
        FountainDocument {
            uri,
            text,
            version,
            parsed: None,
        }
    }

    pub fn parse(&mut self) {
        let config = betterfountain_rust::Conf::default();
        let result = betterfountain_rust::parse(&self.text, &config, false, Some(true));
        self.parsed = Some(ParsedFountain {
            tokens: result.tokens,
            properties: result.properties,
            statistics: result.statistics,
        });
    }
}

#[derive(Clone)]
pub struct DocumentStore {
    pub documents: Arc<Mutex<HashMap<String, FountainDocument>>>,
}

impl DocumentStore {
    pub fn new() -> Self {
        DocumentStore {
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert(&self, uri: String, doc: FountainDocument) {
        let mut docs = self.documents.lock().await;
        docs.insert(uri, doc);
    }

    pub async fn get(&self, uri: &str) -> Option<FountainDocument> {
        let docs = self.documents.lock().await;
        docs.get(uri).cloned()
    }

    pub async fn get_mut(&self, uri: &str) -> Option<FountainDocument> {
        let docs = self.documents.lock().await;
        docs.get(uri).cloned()
    }

    pub async fn get_mut_ref(&self, uri: &str) -> Option<FountainDocument> {
        let docs = self.documents.lock().await;
        docs.get(uri).cloned()
    }

    pub async fn update(&self, uri: String, doc: FountainDocument) {
        let mut docs = self.documents.lock().await;
        docs.insert(uri, doc);
    }

    pub async fn remove(&self, uri: &str) -> Option<FountainDocument> {
        let mut docs = self.documents.lock().await;
        docs.remove(uri)
    }
}

impl Default for DocumentStore {
    fn default() -> Self {
        Self::new()
    }
}