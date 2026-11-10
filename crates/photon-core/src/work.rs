//! Work: Zephium's agent canvas, minimal seam.
//! A Work session is a run an agent (human or model) performs against
//! browsed pages. Results stay on the canvas as artifacts, not chat scroll.

use crate::TabId;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkSession {
    pub id: u64,
    pub prompt: String,
    pub source_tab: Option<TabId>,
    pub artifacts: Vec<String>,
    pub status: WorkStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WorkStatus {
    Draft,
    Running,
    NeedsApproval,
    Done,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkOutcome {
    pub session_id: u64,
    pub summary: String,
    pub artifacts: Vec<String>,
}

impl WorkSession {
    pub fn new(prompt: impl Into<String>, source_tab: Option<TabId>) -> Self {
        Self {
            id: crate::PageId::next().get(),
            prompt: prompt.into(),
            source_tab,
            artifacts: Vec::new(),
            status: WorkStatus::Draft,
        }
    }

    pub fn add_artifact(&mut self, artifact: impl Into<String>) {
        self.artifacts.push(artifact.into());
    }

    pub fn finish(&mut self, summary: impl Into<String>) -> WorkOutcome {
        self.status = WorkStatus::Done;
        WorkOutcome {
            session_id: self.id,
            summary: summary.into(),
            artifacts: self.artifacts.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_session_lifecycle() {
        let mut s = WorkSession::new("plan a trip", None);
        s.add_artifact("table: flights");
        let out = s.finish("done");
        assert_eq!(s.status, WorkStatus::Done);
        assert_eq!(out.artifacts.len(), 1);
    }
}
