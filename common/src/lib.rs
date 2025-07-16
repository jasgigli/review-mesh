use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewSession {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub participants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Comment {
    pub id: String,
    pub session_id: String,
    pub author: String,
    pub file: String,
    pub hunk_id: String,
    pub line: usize,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatLine {
    pub id: String,
    pub session_id: String,
    pub author: String,
    pub body: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffHunk {
    pub id: String,
    pub file: String,
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn chatline_serde_roundtrip() {
        let chat = ChatLine {
            id: "1".to_string(),
            session_id: "sess1".to_string(),
            author: "alice".to_string(),
            body: "Hello!".to_string(),
            created_at: Utc.timestamp_opt(1_600_000_000, 0).unwrap(),
        };
        let json = serde_json::to_string(&chat).unwrap();
        let de: ChatLine = serde_json::from_str(&json).unwrap();
        assert_eq!(chat, de);
    }
}
