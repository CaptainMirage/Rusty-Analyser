use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub title: &'static str,
    pub description: &'static str,
}