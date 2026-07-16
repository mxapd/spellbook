use serde::{Deserialize, Serialize};

#[cfg(feature = "background-jobs")]
use chrono::{DateTime, Utc};
#[cfg(feature = "background-jobs")]
use std::path::PathBuf;

#[cfg(feature = "background-jobs")]
pub type JobId = u64;

#[cfg(feature = "background-jobs")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[cfg(feature = "background-jobs")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub spell_name: String,
    pub command: String,
    pub status: JobStatus,
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_file: PathBuf,
    pub error_file: PathBuf,
}

#[cfg(feature = "background-jobs")]
impl Job {
    pub fn new(id: JobId, spell_name: String, command: String, output_dir: &PathBuf) -> Self {
        let timestamp = Utc::now();
        Self {
            id,
            spell_name,
            command,
            status: JobStatus::Queued,
            pid: None,
            exit_code: None,
            started_at: timestamp,
            completed_at: None,
            output_file: output_dir.join(format!("job_{}.out", id)),
            error_file: output_dir.join(format!("job_{}.err", id)),
        }
    }
}

#[cfg(feature = "recents")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub spell_id: String,
    pub spell_name: String,
    pub timestamp: DateTime<Utc>,
    pub action: RecentAction,
}

#[cfg(feature = "recents")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RecentAction {
    Run,
    Copy,
}

#[cfg(feature = "recents")]
impl RecentEntry {
    pub fn new(spell_id: String, spell_name: String, action: RecentAction) -> Self {
        Self {
            spell_id,
            spell_name,
            timestamp: Utc::now(),
            action,
        }
    }
}

#[cfg(feature = "background-jobs")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobsData {
    pub jobs: Vec<Job>,
    pub next_id: JobId,
}

#[cfg(feature = "background-jobs")]
impl Default for JobsData {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 1,
        }
    }
}

#[cfg(feature = "background-jobs")]
#[derive(Debug, Clone)]
pub struct JobManager {
    pub jobs: Vec<Job>,
    pub next_id: JobId,
    pub max_concurrent: usize,
    pub max_stored: usize,
}

#[cfg(feature = "background-jobs")]
impl Default for JobManager {
    fn default() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 1,
            max_concurrent: 10,
            max_stored: 50,
        }
    }
}

#[cfg(feature = "background-jobs")]
impl JobManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn get_next_id(&mut self) -> JobId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn cleanup_completed(&mut self) {
        self.jobs.retain(|j| {
            !matches!(
                j.status,
                JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
            )
        });
        while self.jobs.len() > self.max_stored {
            self.jobs.remove(0);
        }
    }

    pub fn running_count(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| j.status == JobStatus::Running)
            .count()
    }
}

#[cfg(all(test, feature = "background-jobs"))]
mod job_tests {
    use super::*;

    #[test]
    fn test_job_status_variants() {
        assert!(matches!(JobStatus::Queued, JobStatus::Queued));
        assert!(matches!(JobStatus::Running, JobStatus::Running));
        assert!(matches!(JobStatus::Completed, JobStatus::Completed));
        assert!(matches!(JobStatus::Failed, JobStatus::Failed));
        assert!(matches!(JobStatus::Cancelled, JobStatus::Cancelled));
    }

    #[test]
    fn test_job_manager_next_id() {
        let mut manager = JobManager::new();
        assert_eq!(manager.get_next_id(), 1);
        assert_eq!(manager.get_next_id(), 2);
        assert_eq!(manager.get_next_id(), 3);
    }
}

#[cfg(all(test, feature = "recents"))]
mod recents_tests {
    use super::*;

    #[test]
    fn test_recent_entry_creation() {
        let entry = RecentEntry::new(
            "test-id".to_string(),
            "Test Spell".to_string(),
            RecentAction::Run,
        );
        assert_eq!(entry.spell_id, "test-id");
        assert_eq!(entry.spell_name, "Test Spell");
        assert!(matches!(entry.action, RecentAction::Run));
    }
}
