use tower_lsp::lsp_types::*;
use crate::parser::FountainDocument;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct CompletionProvider {
    documents: Arc<Mutex<HashMap<String, FountainDocument>>>,
}

impl CompletionProvider {
    pub fn new(documents: Arc<Mutex<HashMap<String, FountainDocument>>>) -> Self {
        CompletionProvider { documents }
    }

    pub async fn provide_completion(
        &self,
        params: CompletionParams,
    ) -> Option<CompletionList> {
        let documents = self.documents.lock().await;
        let uri = params.text_document_position.text_document.uri.to_string();
        
        let doc = match documents.get(&uri) {
            Some(doc) => doc,
            None => return None,
        };

        let current_line = params.text_document_position.position.line;
        let current_char = params.text_document_position.position.character;
        
        let lines: Vec<&str> = doc.text.lines().collect();
        let line_text = lines.get(current_line as usize).map(|s| *s).unwrap_or("");
        let line_trimmed = line_text.trim();
        
        if doc.parsed.is_none() {
            return None;
        }
        
        let parsed = doc.parsed.as_ref().unwrap();
        let first_title_line = parsed.properties.first_token_line.unwrap_or(0);
        
        let prev_line = if current_line > 0 {
            lines.get((current_line - 1) as usize).map(|s| *s).unwrap_or("")
        } else {
            ""
        };
        let prev_line_empty = prev_line.trim().is_empty();
        
        eprintln!("[Completion] characters: {:?}", parsed.properties.characters.keys().collect::<Vec<_>>());
        
        let mut items: Vec<CompletionItem> = Vec::new();
        
        if first_title_line as i32 >= current_line as i32 {
            self.add_title_page_completions(line_trimmed, current_char, &mut items, &parsed.properties);
        } else {
            self.add_scene_completions(line_trimmed, current_char, current_line, &mut items);
            self.add_transition_completions(line_trimmed, current_char, current_line, prev_line_empty, &mut items);
            self.add_character_completions(line_trimmed, current_char, current_line, &mut items, &parsed.properties, &doc.text);
            self.add_scene_heading_completions(line_trimmed, current_char, current_line, &mut items, &parsed.properties);
            self.add_location_completions_after_scene_heading(line_trimmed, current_char, current_line, line_text, &mut items, &parsed.properties);
            self.add_note_completions(line_text, current_char, current_line, &mut items);
            self.add_parenthetical_completions(line_trimmed, current_char, current_line, line_text, &mut items, &parsed.properties);
            self.add_underline_completions(line_text, current_char, current_line, &mut items);
            self.add_scene_number_completions(line_trimmed, line_text, current_char, current_line, &mut items, &parsed.properties);
        }

        if items.is_empty() {
            return None;
        }

        Some(CompletionList {
            items,
            is_incomplete: false,
        })
    }

    fn add_title_page_completions(
        &self,
        line_trimmed: &str,
        _current_char: u32,
        items: &mut Vec<CompletionItem>,
        properties: &betterfountain_rust::ScreenplayProperties,
    ) {
        if line_trimmed.is_empty() {
            let title_keys = &["Title", "Credit", "Author", "Source", "Notes", "Draft Date", "Date", "Contact", "Copyright", "Watermark", "Font", "Font Italic", "Font Bold", "Font Bold Italic", "Metadata", "Revision", "TL", "TC", "TR", "CC", "BL", "BR", "Header", "Footer"];
            let title_details = &["The title of the screenplay", "How the author is credited", "The name of the author", "An additional source for the screenplay", "Additional notes", "The date of the current draft", "The date of the screenplay", "Contact details", "Copyright information", "A watermark displayed on every page", "The font used in the screenplay", "The italic font used in the screenplay", "The bold font used in the screenplay", "The bold italic font used in the screenplay", "Metadata json string", "The name of the current and past revisions", "Top Left", "Top Center", "Top Right", "Center Center", "Bottom Left", "Bottom Right", "Header used throughout the document", "Footer used throughout the document"];
            
            for (i, key) in title_keys.iter().enumerate() {
                if !properties.title_keys.contains(&key.to_lowercase()) {
                    items.push(CompletionItem {
                        label: format!("{}:", key),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(title_details.get(i).unwrap_or(&"").to_string()),
                        sort_text: Some(format!("{:02}", i)),
                        ..Default::default()
                    });
                }
            }
        } else {
            let current_key = line_trimmed.to_lowercase();
            if current_key == "title:" {
                items.push(CompletionItem {
                    label: "**《》**".to_string(),
                    kind: Some(CompletionItemKind::SNIPPET),
                    insert_text: Some("**《$1》**".to_string()),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                });
            } else if current_key == "author:" || current_key == "author" {
                items.push(CompletionItem {
                    label: "Author Name".to_string(),
                    kind: Some(CompletionItemKind::TEXT),
                    ..Default::default()
                });
            } else if current_key == "date:" || current_key == "draft date:" {
                let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                items.push(CompletionItem {
                    label: today,
                    kind: Some(CompletionItemKind::TEXT),
                    sort_text: Some("0A".to_string()),
                    ..Default::default()
                });
            } else if current_key == "credit:" {
                items.push(CompletionItem {
                    label: "Written by".to_string(),
                    kind: Some(CompletionItemKind::TEXT),
                    sort_text: Some("0A".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "By".to_string(),
                    kind: Some(CompletionItemKind::TEXT),
                    sort_text: Some("0B".to_string()),
                    ..Default::default()
                });
            } else if current_key == "source:" {
                items.push(CompletionItem {
                    label: "Based on".to_string(),
                    kind: Some(CompletionItemKind::TEXT),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "Story by".to_string(),
                    kind: Some(CompletionItemKind::TEXT),
                    ..Default::default()
                });
            } else if current_key == "copyright:" {
                let year = chrono::Local::now().format("%Y").to_string();
                items.push(CompletionItem {
                    label: format!("(c) {}", year),
                    kind: Some(CompletionItemKind::TEXT),
                    ..Default::default()
                });
            }
        }
    }

    fn add_scene_completions(
        &self,
        line_trimmed: &str,
        current_char: u32,
        current_line: u32,
        items: &mut Vec<CompletionItem>,
    ) {
        if line_trimmed == "." || line_trimmed == ".(" || line_trimmed == "。" || line_trimmed == "。(" {
            let trigger_len = 1u32;
            let range = Range {
                start: Position { line: current_line, character: current_char - trigger_len },
                end: Position { line: current_line, character: current_char },
            };
            
            items.push(CompletionItem {
                label: ".(内景)".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: ".(内景)".to_string() })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("内景".to_string()),
                sort_text: Some("00B".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ".(外景)".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: ".(外景)".to_string() })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("外景".to_string()),
                sort_text: Some("00C".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ".(内外景)".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: ".(内外景)".to_string() })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("内外景".to_string()),
                sort_text: Some("00D".to_string()),
                ..Default::default()
            });
        }
    }

    fn add_location_completions_after_scene_heading(
        &self,
        line_trimmed: &str,
        current_char: u32,
        current_line: u32,
        line_text: &str,
        items: &mut Vec<CompletionItem>,
        properties: &betterfountain_rust::ScreenplayProperties,
    ) {
        let is_scene_line = |line: &str| -> bool {
            let upper = line.to_uppercase();
            upper.starts_with("INT.") || upper.starts_with("EXT.") || 
            upper.starts_with("INT/EXT.") || upper.starts_with("EST.") ||
            upper.starts_with("I/E.") || (line.trim().starts_with('.') && line.trim().len() > 1)
        };

        if is_scene_line(line_trimmed) && !line_trimmed.ends_with('-') && !line_trimmed.ends_with('–') && !line_trimmed.ends_with('—') {
            if current_char > 0 && line_text.chars().nth((current_char - 1) as usize) == Some(' ') {
                for location in properties.locations.keys() {
                    if !location.is_empty() {
                        items.push(CompletionItem {
                            label: location.clone(),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some("场景位置".to_string()),
                            sort_text: Some(format!("0C{}", location)),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }

    fn add_transition_completions(
        &self,
        line_trimmed: &str,
        current_char: u32,
        current_line: u32,
        prev_line_empty: bool,
        items: &mut Vec<CompletionItem>,
    ) {
        if line_trimmed == ">" || line_trimmed == "＞" || line_trimmed == "》" {
            let trigger_len = 1u32;
            let range = Range {
                start: Position { line: current_line, character: current_char - trigger_len },
                end: Position { line: current_line, character: current_char },
            };
            let prefix = if prev_line_empty { "" } else { "\n" };
            
            items.push(CompletionItem {
                label: ">".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: ">".to_string() })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0A".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">叠化".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>叠化", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0b".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">淡出淡入".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>淡出淡入", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0c".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">切到".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>切到", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0d".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">闪回".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>闪回", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0f".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">淡出".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>淡出", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0g".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">淡入".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>淡入", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0h".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">闪回结束".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>闪回结束", prefix) })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入转场".to_string()),
                sort_text: Some("0i".to_string()),
                ..Default::default()
            });
            let shot_cut_1 = if prev_line_empty { ">{=镜头交切=} ↓".to_string() } else { "\n>{=镜头交切=} ↓".to_string() };
            let shot_cut_2 = if prev_line_empty { ">{#镜头交切#} ↓".to_string() } else { "\n>{#镜头交切#} ↓".to_string() };
            let shot_cut_3 = if prev_line_empty { ">{+镜头交切+} ↓".to_string() } else { "\n>{+镜头交切+} ↓".to_string() };
            let shot_cut_end = if prev_line_empty { ">{-结束交切-} ↑".to_string() } else { "\n>{-结束交切-} ↑".to_string() };

            items.push(CompletionItem {
                label: ">{=镜头交切=} ↓ (只含以后新场景)".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: shot_cut_1 })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入镜头交切 (只含以后新场景)".to_string()),
                sort_text: Some("0j".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">{#镜头交切#} ↓ (含前一.当前.以后场景)".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: shot_cut_2 })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入镜头交切（含前一.当前.以后场景）".to_string()),
                sort_text: Some("0k".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">{+镜头交切+} ↓ (含当前.以后场景)".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: shot_cut_3 })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入镜头交切（含当前.以后场景）".to_string()),
                sort_text: Some("0k1".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: ">{-结束交切-} ↑".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: shot_cut_end })),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入结束交切".to_string()),
                sort_text: Some("0l".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: "> <".to_string(),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!("{}>$1<", prefix) })),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("插入居中语法".to_string()),
                sort_text: Some("1B".to_string()),
                ..Default::default()
            });
        }
    }

    fn add_character_completions(
        &self,
        line_trimmed: &str,
        current_char: u32,
        current_line: u32,
        items: &mut Vec<CompletionItem>,
        properties: &betterfountain_rust::ScreenplayProperties,
        text: &str,
    ) {
        tracing::info!("add_character_completions called, line_trimmed: '{}', characters count: {}", line_trimmed, properties.characters.len());
        
        if line_trimmed == "@" {
            let characters: Vec<&String> = properties.characters.keys().collect();
            tracing::info!("Characters found: {:?}", characters);
            
            let mut current_scene_characters: Vec<&String> = Vec::new();
            let lines: Vec<&str> = text.lines().collect();
            
            for (line_idx, line) in lines.iter().enumerate() {
                if line_idx as u32 >= current_line {
                    break;
                }
                let trimmed = line.trim();
                if trimmed.starts_with('.') || trimmed.to_uppercase().starts_with("INT.") || trimmed.to_uppercase().starts_with("EXT.") {
                    break;
                }
                for char_name in properties.characters.keys() {
                    if trimmed == char_name.to_uppercase() || trimmed.starts_with(&format!("{} ", char_name.to_uppercase())) {
                        if !current_scene_characters.contains(&char_name) {
                            current_scene_characters.push(char_name);
                        }
                    }
                }
            }

            let mut index = 0;
            for char_name in &current_scene_characters {
                items.push(CompletionItem {
                    label: format!("@{}", char_name),
                    insert_text: Some(char_name.to_string()),
                    kind: Some(CompletionItemKind::KEYWORD),
                    detail: Some("当前场景角色".to_string()),
                    sort_text: Some(format!("0A{:03}", index)),
                    ..Default::default()
                });
                index += 1;
            }

            let sort_text = if current_scene_characters.is_empty() {
                "0A".to_string()
            } else {
                "2".to_string()
            };
            
            for char_name in &characters {
                if !current_scene_characters.contains(char_name) && !char_name.trim().is_empty() {
                    items.push(CompletionItem {
                        label: format!("@{}", char_name),
                        insert_text: Some(char_name.to_string()),
                        kind: Some(CompletionItemKind::TEXT),
                        detail: Some("角色".to_string()),
                        sort_text: Some(format!("{}{}", sort_text, char_name)),
                        ..Default::default()
                    });
                }
            }
        }

        if (line_trimmed == "e" || line_trimmed == "E") && current_char == 1 {
            items.push(CompletionItem {
                label: "EXT.".to_string(),
                insert_text: Some("EXT.".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("外景".to_string()),
                sort_text: Some("001F".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: "INT/EXT.".to_string(),
                insert_text: Some("INT/EXT.".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("内/外景".to_string()),
                sort_text: Some("001h".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: "EST. ".to_string(),
                insert_text: Some("EST. ".to_string()),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("建立镜头".to_string()),
                sort_text: Some("001i".to_string()),
                ..Default::default()
            });
        }

        if (line_trimmed == "i" || line_trimmed == "I") && current_char == 1 {
            items.push(CompletionItem {
                label: "INT.".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("内景".to_string()),
                sort_text: Some("001F".to_string()),
                ..Default::default()
            });
            items.push(CompletionItem {
                label: "INT/EXT.".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("内/外景".to_string()),
                sort_text: Some("001h".to_string()),
                ..Default::default()
            });
        }
    }

    fn add_scene_heading_completions(
        &self,
        line_trimmed: &str,
        current_char: u32,
        current_line: u32,
        items: &mut Vec<CompletionItem>,
        properties: &betterfountain_rust::ScreenplayProperties,
    ) {
        let is_scene_line = |line: &str| -> bool {
            let upper = line.to_uppercase();
            upper.starts_with("INT.") || upper.starts_with("EXT.") || 
            upper.starts_with("INT/EXT.") || upper.starts_with("EST.") ||
            upper.starts_with("I/E.") || (line.trim().starts_with('.') && line.trim().len() > 1)
        };

        if is_scene_line(line_trimmed) {
            if line_trimmed.ends_with('-') || line_trimmed.ends_with('–') || line_trimmed.ends_with('—') {
                let range = Range {
                    start: Position { line: current_line, character: current_char - 1 },
                    end: Position { line: current_line, character: current_char },
                };
                items.push(CompletionItem {
                    label: "- 日".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - 日".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("A".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- 夜".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - 夜".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("B".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- 黎明".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - 黎明".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("BA".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- 清晨".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - 清晨".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("BB".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- 黄昏".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - 黄昏".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("BC".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- DAY".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - DAY".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("E".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- NIGHT".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - NIGHT".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("F".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- DUSK".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - DUSK".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("G".to_string()),
                    ..Default::default()
                });
                items.push(CompletionItem {
                    label: "- DAWN".to_string(),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " - DAWN".to_string() })),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some("时间".to_string()),
                    sort_text: Some("H".to_string()),
                    ..Default::default()
                });
            }
        }
    }

    fn add_note_completions(
        &self,
        line_text: &str,
        current_char: u32,
        current_line: u32,
        items: &mut Vec<CompletionItem>,
    ) {
        if current_char > 0 {
            let char_idx = (current_char - 1) as usize;
            if char_idx < line_text.len() {
                let prev_char = line_text.chars().nth(char_idx).unwrap_or(' ');
                if prev_char == '【' || prev_char == '[' {
                    let trigger_len = 1u32;
                    let range = Range {
                        start: Position { line: current_line, character: current_char - trigger_len },
                        end: Position { line: current_line, character: current_char },
                    };
                    items.push(CompletionItem {
                        label: "[[ ]] 插入note".to_string(),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("插入note".to_string()),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: "[[$1]]".to_string() })),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        sort_text: Some("0B".to_string()),
                        ..Default::default()
                    });
                    items.push(CompletionItem {
                        label: "[[| ]] 插入note(强制原位)".to_string(),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("插入note(强制原位)".to_string()),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: "[[|$1]]".to_string() })),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        sort_text: Some("0C".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    fn add_parenthetical_completions(
        &self,
        line_trimmed: &str,
        current_char: u32,
        current_line: u32,
        line_text: &str,
        items: &mut Vec<CompletionItem>,
        properties: &betterfountain_rust::ScreenplayProperties,
    ) {
        let is_character_line = |line: &str, props: &betterfountain_rust::ScreenplayProperties| -> bool {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return false;
            }
            if trimmed.starts_with('@') {
                return true;
            }
            let trimmed_upper = trimmed.to_uppercase();
            for char_name in props.characters.keys() {
                let char_upper = char_name.to_uppercase();
                if trimmed_upper == char_upper || trimmed_upper.starts_with(&format!("{} ", char_upper)) {
                    return true;
                }
            }
            false
        };

        let is_prev_line_character = |line: u32, text: &str, props: &betterfountain_rust::ScreenplayProperties| -> bool {
            if line == 0 {
                return false;
            }
            let lines: Vec<&str> = text.lines().collect();
            if let Some(prev_line) = lines.get((line - 1) as usize) {
                is_character_line(prev_line, props)
            } else {
                false
            }
        };

        let current_is_character = is_character_line(line_trimmed, properties);
        let prev_is_character = is_prev_line_character(current_line, line_text, properties);
        
        if current_char > 0 {
            let char_idx = (current_char - 1) as usize;
            if char_idx < line_text.len() {
                let prev_char = line_text.chars().nth(char_idx).unwrap_or(' ');
                let range = Range {
                    start: Position { line: current_line, character: current_char - 1 },
                    end: Position { line: current_line, character: current_char },
                };
                
                if prev_char == '（' {
                    items.push(CompletionItem {
                        label: "() 转为英文括号".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " ($1)".to_string() })),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("转换为英文括号".to_string()),
                        sort_text: Some("0A".to_string()),
                        ..Default::default()
                    });
                    if current_is_character || prev_is_character {
                        items.push(CompletionItem {
                            label: "(画外音)".to_string(),
                            text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " (画外音)".to_string() })),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some("画外音".to_string()),
                            sort_text: Some("0B".to_string()),
                            ..Default::default()
                        });
                        items.push(CompletionItem {
                            label: "(旁白)".to_string(),
                            text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " (旁白)".to_string() })),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some("旁白".to_string()),
                            sort_text: Some("0C".to_string()),
                            ..Default::default()
                        });
                    }
                } else if (current_is_character || prev_is_character) && prev_char == '(' && !line_trimmed.starts_with(".(") {
                    items.push(CompletionItem {
                        label: "(画外音)".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " (画外音)".to_string() })),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some("画外音".to_string()),
                        sort_text: Some("0A".to_string()),
                        ..Default::default()
                    });
                    items.push(CompletionItem {
                        label: "(旁白)".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " (旁白)".to_string() })),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some("旁白".to_string()),
                        sort_text: Some("1A".to_string()),
                        ..Default::default()
                    });
                    items.push(CompletionItem {
                        label: "(O.S.)".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " (O.S.)".to_string() })),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some("画外音".to_string()),
                        sort_text: Some("1B".to_string()),
                        ..Default::default()
                    });
                    items.push(CompletionItem {
                        label: "(V.O.)".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " (V.O.)".to_string() })),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some("旁白".to_string()),
                        sort_text: Some("1C".to_string()),
                        ..Default::default()
                    });
                    items.push(CompletionItem {
                        label: "()".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " ($1)".to_string() })),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("添加对话说明".to_string()),
                        sort_text: Some("3B".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    fn add_underline_completions(
        &self,
        line_text: &str,
        current_char: u32,
        current_line: u32,
        items: &mut Vec<CompletionItem>,
    ) {
        if current_char >= 2 {
            let char_idx = (current_char - 2) as usize;
            if char_idx < line_text.len() {
                let prev_chars: String = line_text.chars().skip(char_idx.saturating_sub(1)).take(2).collect();
                if prev_chars == "——" || prev_chars == "--" {
                    let range = Range {
                        start: Position { line: current_line, character: current_char - 2 },
                        end: Position { line: current_line, character: current_char },
                    };
                    items.push(CompletionItem {
                        label: "_     转为下划线".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: "_".to_string() })),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some("转为下划线".to_string()),
                        sort_text: Some("0A".to_string()),
                        ..Default::default()
                    });
                    items.push(CompletionItem {
                        label: "_ _   插入下划线语法".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: "_$1_".to_string() })),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        kind: Some(CompletionItemKind::SNIPPET),
                        detail: Some("插入下划线语法".to_string()),
                        sort_text: Some("0B".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    fn add_scene_number_completions(
        &self,
        line_trimmed: &str,
        line_text: &str,
        current_char: u32,
        current_line: u32,
        items: &mut Vec<CompletionItem>,
        properties: &betterfountain_rust::ScreenplayProperties,
    ) {
        if current_char > 0 {
            let char_idx = (current_char - 1) as usize;
            if char_idx < line_text.len() {
                let prev_char = line_text.chars().nth(char_idx).unwrap_or(' ');
                if prev_char == '#' {
                    let is_scene_line = |line: &str| -> bool {
                        let upper = line.to_uppercase();
                        upper.starts_with("INT.") || upper.starts_with("EXT.") || 
                        upper.starts_with("INT/EXT.") || upper.starts_with("EST.") ||
                        upper.starts_with("I/E.") || (line.trim().starts_with('.') && line.trim().len() > 1)
                    };
                    
                    if !is_scene_line(line_trimmed) {
                        return;
                    }
                    
                    let range = Range {
                        start: Position { line: current_line, character: current_char - 1 },
                        end: Position { line: current_line, character: current_char },
                    };
                    items.push(CompletionItem {
                        label: "#${}#".to_string(),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: " #${}#".to_string() })),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        kind: Some(CompletionItemKind::VARIABLE),
                        detail: Some("插入场号".to_string()),
                        sort_text: Some("0A".to_string()),
                        ..Default::default()
                    });

                    if let Some(ref vars) = properties.scene_number_vars {
                        for var in vars {
                            items.push(CompletionItem {
                                label: format!("#{{{}}}#", var),
                                text_edit: Some(CompletionTextEdit::Edit(TextEdit { range, new_text: format!(" #{{{}}}#", var) })),
                                kind: Some(CompletionItemKind::VARIABLE),
                                detail: Some("Scene number".to_string()),
                                sort_text: Some(format!("0B{}", var)),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }
    }
}