use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use shared::Command;

use crate::journal::JournalId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalCommand {
    Create(JournalCommandCreate),
    Update(JournalCommandUpdate),
    Delete(HashSet<JournalId>),
}

impl Command for JournalCommand {
    fn command_type(&self) -> &'static str {
        match self {
            JournalCommand::Create(command) => command.command_type(),
            JournalCommand::Update(command) => command.command_type(),
            JournalCommand::Delete(_) => "whiterabbit::command::JournalCommandDelete",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCommandCreate {
    pub name: String,
    pub description: String,
    pub tags: HashSet<String>,
}

impl Command for JournalCommandCreate {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::JournalCommandCreate"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalCommandUpdate {
    pub id: JournalId,
    pub name: String,
    pub description: Option<String>,
    pub tags: Option<HashSet<String>>,
}

impl Command for JournalCommandUpdate {
    fn command_type(&self) -> &'static str {
        "whiterabbit::command::JournalCommandUpdate"
    }
}
