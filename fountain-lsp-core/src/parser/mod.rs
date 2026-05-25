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
        let mut root_children: Vec<DocumentSymbol> = Vec::new();

        for token in &self.properties.structure {
            if token.isnote {
                root_children.push(self.create_note_symbol(token));
                continue;
            }
            if token.is_bookmark {
                root_children.push(self.create_bookmark_symbol(token));
                continue;
            }
            let mut symbol = self.token_to_document_symbol(token);
            let mut children: Vec<DocumentSymbol> = Vec::new();

            self.build_children_tree(&token.children, &mut children);
            self.build_children_tree(&token.structs, &mut children);

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
        root_children
    }

    // 递归构建子节点树，保留 note/bookmark 在原位置
    fn build_children_tree(
        &self,
        tokens: &[betterfountain_rust::StructToken],
        parent_children: &mut Vec<DocumentSymbol>,
    ) {
        for token in tokens {
            // 子节点中的 note/bookmark 保留在原位置作为 children
            if token.isnote {
                parent_children.push(self.create_note_symbol(token));
                continue;
            }
            if token.is_bookmark {
                parent_children.push(self.create_bookmark_symbol(token));
                continue;
            }
            
            let mut symbol = self.token_to_document_symbol(token);
            let mut children: Vec<DocumentSymbol> = Vec::new();

            // 递归处理孙节点
            self.build_children_tree(&token.children, &mut children);
            self.build_children_tree(&token.structs, &mut children);

            // 添加 synopses
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
        let duration = if token.section {
            // section 使用子节点时长相加
            self.format_duration(self.calculate_section_duration(token))
        } else {
            // 场景和其他使用自己的时长
            self.format_duration(token.duration_sec)
        };
        
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

        let name_with_duration = if duration.is_empty() {
            name
        } else {
            format!("{} [{}]", name, duration)
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

        // children 由 build_children_tree 处理
        let children: Option<Vec<DocumentSymbol>> = None;

        DocumentSymbol {
            name: name_with_duration,
            detail: None,
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