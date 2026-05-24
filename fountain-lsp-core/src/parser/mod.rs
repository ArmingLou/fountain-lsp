use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct ParsedFountain {
    pub tokens: Vec<betterfountain_rust::ScriptToken>,
    pub properties: betterfountain_rust::ScreenplayProperties,
    pub statistics: Option<betterfountain_rust::statistics::Statistics>,
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