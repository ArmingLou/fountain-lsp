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
        // note 和 bookmark 统一收集器，最后抽取为根节点
        let mut notes_collector: Vec<DocumentSymbol> = Vec::new();
        let mut bookmarks_collector: Vec<DocumentSymbol> = Vec::new();

        for token in &self.properties.structure {
            if token.isnote {
                notes_collector.push(self.create_note_symbol(token));
                continue;
            }
            if token.is_bookmark {
                bookmarks_collector.push(self.create_bookmark_symbol(token));
                continue;
            }
            let mut symbol = self.token_to_document_symbol(token);
            let mut children: Vec<DocumentSymbol> = Vec::new();

            self.build_children_tree(&token.children, &mut children, &mut notes_collector, &mut bookmarks_collector);
            self.build_children_tree(&token.structs, &mut children, &mut notes_collector, &mut bookmarks_collector);

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

            // 附属注解统一推送到收集器
            for note in &token.notes {
                notes_collector.push(self.create_note_item_symbol(&note.note, note.line as u32));
            }

            if !children.is_empty() {
                children.sort_by_key(|s| s.selection_range.start.line);
                symbol.children = Some(children);
            }

            root_children.push(symbol);
        }

        // NOTES/Bookmarks 置于顶层，子节点保留真实 range 确保跳转正确
        let top_range = Range::new(Position::new(0, 0), Position::new(0, 0));

        if !notes_collector.is_empty() {
            notes_collector.sort_by_key(|s| s.selection_range.start.line);
            root_children.push(DocumentSymbol {
                name: "NOTES".to_string(),
                detail: None,
                kind: SymbolKind::NAMESPACE,
                tags: None,
                deprecated: None,
                range: top_range,
                selection_range: top_range,
                children: Some(notes_collector),
            });
        }

        if !bookmarks_collector.is_empty() {
            bookmarks_collector.sort_by_key(|s| s.selection_range.start.line);
            root_children.push(DocumentSymbol {
                name: "Bookmarks".to_string(),
                detail: None,
                kind: SymbolKind::NAMESPACE,
                tags: None,
                deprecated: None,
                range: top_range,
                selection_range: top_range,
                children: Some(bookmarks_collector),
            });
        }

        root_children
    }

    // 递归构建子节点树，note/bookmark 统一推送到收集器
    fn build_children_tree(
        &self,
        tokens: &[betterfountain_rust::StructToken],
        parent_children: &mut Vec<DocumentSymbol>,
        notes_collector: &mut Vec<DocumentSymbol>,
        bookmarks_collector: &mut Vec<DocumentSymbol>,
    ) {
        for token in tokens {
            // note/bookmark 推送到收集器，不放在子节点中
            if token.isnote {
                notes_collector.push(self.create_note_symbol(token));
                continue;
            }
            if token.is_bookmark {
                bookmarks_collector.push(self.create_bookmark_symbol(token));
                continue;
            }
            
            let mut symbol = self.token_to_document_symbol(token);
            let mut children: Vec<DocumentSymbol> = Vec::new();

            // 递归处理孙节点
            self.build_children_tree(&token.children, &mut children, notes_collector, bookmarks_collector);
            self.build_children_tree(&token.structs, &mut children, notes_collector, bookmarks_collector);

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

            // 附属注解统一推送到收集器
            for note in &token.notes {
                notes_collector.push(self.create_note_item_symbol(&note.note, note.line as u32));
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

    // 创建基于文本+行号的符号条目（用于 token.notes 中的附属注解）
    fn create_note_item_symbol(&self, text: &str, line: u32) -> DocumentSymbol {
        let range = Range::new(
            Position::new(line, 0),
            Position::new(line, text.len() as u32)
        );
        DocumentSymbol {
            name: if text.is_empty() { format!("Line {}", line) } else { text.to_string() },
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_fountain(text: &str) -> ParsedFountain {
        let config = betterfountain_rust::Conf::default();
        let result = betterfountain_rust::parse(text, &config, false, Some(true));
        ParsedFountain {
            tokens: result.tokens,
            properties: result.properties,
            statistics: result.statistics,
        }
    }

    #[test]
    fn test_notes_extracted_to_root() {
        let text = "\
INT. ROOM - DAY

JOHN
Hello world!

[[这是一条笔记]]

EXT. STREET - NIGHT

MARY
Hi there!

/*| 章节目标：完成第一幕 */

# Act 1
## Scene 1
>第一幕第一场景 <
";

        let parsed = parse_fountain(text);
        let symbols = parsed.to_document_symbols();

        eprintln!("=== 大纲节点总数: {} ===", symbols.len());
        for (i, s) in symbols.iter().enumerate() {
            let children_count = s.children.as_ref().map(|c| c.len()).unwrap_or(0);
            eprintln!("  [{}] name={:?} kind={:?} range={:?} children={}", i, s.name, s.kind, s.range.start.line, children_count);
            if let Some(children) = &s.children {
                for (j, c) in children.iter().enumerate() {
                    eprintln!("    child[{}] name={:?} kind={:?} range={:?}", j, c.name, c.kind, c.range.start.line);
                }
            }
        }

        // 验证 NOTES 节点存在且包含笔记
        let notes_node = symbols.iter().find(|s| s.name == "NOTES");
        assert!(notes_node.is_some(), "应该存在 NOTES 根节点");
        let notes_node = notes_node.unwrap();
        assert!(notes_node.children.is_some(), "NOTES 根节点应该有子节点");
        let notes_children = notes_node.children.as_ref().unwrap();
        assert!(!notes_children.is_empty(), "NOTES 根节点的子节点不应为空");
        assert!(notes_children.iter().any(|c| c.name == "这是一条笔记"),
            "NOTES 下应包含 '这是一条笔记'");

        // 验证 Bookmarks 节点存在且包含书签
        let bookmarks_node = symbols.iter().find(|s| s.name == "Bookmarks");
        assert!(bookmarks_node.is_some(), "应该存在 Bookmarks 根节点");
        let bookmarks_node = bookmarks_node.unwrap();
        assert!(bookmarks_node.children.is_some(), "Bookmarks 根节点应该有子节点");
        let bookmark_children = bookmarks_node.children.as_ref().unwrap();
        assert!(!bookmark_children.is_empty(), "Bookmarks 根节点的子节点不应为空");
        assert!(bookmark_children.iter().any(|c| c.name == "章节目标：完成第一幕"),
            "Bookmarks 下应包含 '章节目标：完成第一幕'");

        // 验证 note/bookmark 不在章节子节点中（已被抽取）
        for s in &symbols {
            if s.name == "NOTES" || s.name == "Bookmarks" {
                continue;
            }
            if let Some(children) = &s.children {
                for c in children {
                    assert_ne!(c.kind, SymbolKind::CONSTANT,
                        "章节子节点中不应再有 CONSTANT 类型: name={:?} parent={:?}", c.name, s.name);
                }
            }
        }

        // 验证 NOTES/Bookmarks 根节点在最顶层（range=0）
        assert_eq!(notes_node.range.start.line, 0,
            "NOTES 应为顶层节点 range=0");
        assert_eq!(bookmarks_node.range.start.line, 0,
            "Bookmarks 应为顶层节点 range=0");

        let notes_child = notes_children.first().unwrap();
        assert_eq!(notes_child.range.start.line, 5,
            "note 子节点 range 应保留真实行号 5，实为 {}", notes_child.range.start.line);
        assert_eq!(notes_child.selection_range.start.line, 5,
            "note 子节点 selection_range 应保留真实行号");

        let bookmark_child = bookmark_children.first().unwrap();
        assert_eq!(bookmark_child.range.start.line, 12,
            "bookmark 子节点 range 应保留真实行号 12，实为 {}", bookmark_child.range.start.line);
        assert_eq!(bookmark_child.selection_range.start.line, 12,
            "bookmark 子节点 selection_range 应保留真实行号");

        eprintln!("测试通过！");
    }
}