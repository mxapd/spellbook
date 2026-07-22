use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::invoker::exec::kill_process;
use crate::log_info;

const MAX_CONCURRENT_JOBS: usize = 10;
const SPELLBOOK_DIR: &str = ".spellbook";
const JOBS_FILE: &str = "jobs.toml";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    #[serde(rename = "queued")]
    Queued,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl Default for JobStatus {
    fn default() -> Self {
        JobStatus::Queued
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: u64,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub status: JobStatus,
    #[serde(skip)]
    pub pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output_file: String,
    pub error_file: String,
    #[serde(default)]
    pub working_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRegistry {
    #[serde(rename = "jobs")]
    pub job_list: Vec<Job>,
    #[serde(rename = "next_id")]
    pub next_id: u64,
}

impl Default for JobRegistry {
    fn default() -> Self {
        Self {
            job_list: Vec::new(),
            next_id: 1,
        }
    }
}

#[derive(Debug)]
pub enum ExecutorError {
    JobLimitReached,
    JobNotFound(u64),
    ShellNotFound,
    IoError(String),
    ConfirmationRequired(String),
}

impl std::fmt::Display for ExecutorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutorError::JobLimitReached => {
                write!(f, "Job limit reached (max {} concurrent jobs)", MAX_CONCURRENT_JOBS)
            }
            ExecutorError::JobNotFound(id) => write!(f, "Job {} not found", id),
            ExecutorError::ShellNotFound => write!(f, "No suitable shell found"),
            ExecutorError::IoError(msg) => write!(f, "IO error: {}", msg),
            ExecutorError::ConfirmationRequired(name) => {
                write!(f, "Confirmation required for: {}", name)
            }
        }
    }
}

impl std::error::Error for ExecutorError {}

// ============================================================================
// JobManager
// ============================================================================

pub struct JobManager {
    registry: Arc<Mutex<JobRegistry>>,
    spool_dir: PathBuf,
    shell_path: String,
    launch_dir: String,
}

impl JobManager {
    pub fn new(launch_dir: String) -> Self {
        let spool_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(SPELLBOOK_DIR);

        if !spool_dir.exists() {
            let _ = fs::create_dir_all(&spool_dir);
        }

        let registry = Self::load_registry(&spool_dir);
        let shell_path = Self::detect_shell();

        let manager = Self {
            registry: Arc::new(Mutex::new(registry)),
            spool_dir,
            shell_path,
            launch_dir,
        };

        manager.start_polling_thread();

        manager
    }

    fn start_polling_thread(&self) {
        let registry = Arc::clone(&self.registry);
        let spool_dir = self.spool_dir.clone();

        thread::spawn(move || {
            loop {
                thread::sleep(POLL_INTERVAL);

                let jobs_to_check: Vec<(u64, Option<u32>)> = {
                    let reg = registry.lock().unwrap();
                    reg.job_list
                        .iter()
                        .filter(|j| j.status == JobStatus::Running)
                        .map(|j| (j.id, j.pid))
                        .collect()
                };

                let mut changed = false;
                let spool_clone = spool_dir.clone();

                for (job_id, pid) in jobs_to_check {
                    if let Some(pid) = pid {
                        let running =
                            match Command::new("ps").arg("-p").arg(pid.to_string()).output() {
                                Ok(output) => {
                                    let output_str = String::from_utf8_lossy(&output.stdout);
                                    output_str.lines().count() > 1
                                }
                                Err(_) => false,
                            };

                        if !running {
                            let mut reg = registry.lock().unwrap();
                            if let Some(j) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                                j.status = JobStatus::Completed;
                                j.exit_code = Some(0);
                                j.completed_at = Some(Utc::now());
                                changed = true;
                                let job_clone = j.clone();
                                drop(reg);
                                Self::send_notification_internal(&job_clone, &spool_clone);
                            }
                        }
                    }
                }

                if changed {
                    let reg = registry.lock().unwrap();
                    let jobs_path = spool_dir.join(JOBS_FILE);
                    if let Ok(contents) = toml::to_string_pretty(&*reg) {
                        let _ = fs::write(&jobs_path, contents);
                    }
                }
            }
        });
    }

    fn load_registry(spool_dir: &PathBuf) -> JobRegistry {
        let jobs_path = spool_dir.join(JOBS_FILE);
        if jobs_path.exists() {
            match fs::read_to_string(&jobs_path) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(registry) => {
                        let mut reg: JobRegistry = registry;
                        reg.cleanup_completed();
                        return reg;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse jobs.toml: {}, using empty registry", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to read jobs.toml: {}, using empty registry", e);
                }
            }
        }
        JobRegistry::default()
    }

    fn save_registry(&self) {
        let registry = self.registry.lock().unwrap();
        let jobs_path = self.spool_dir.join(JOBS_FILE);
        match toml::to_string_pretty(&*registry) {
            Ok(contents) => {
                if let Err(e) = fs::write(&jobs_path, contents) {
                    eprintln!("Failed to save jobs.toml: {}", e);
                }
            }
            Err(e) => {
                eprintln!("Failed to serialize jobs.toml: {}", e);
            }
        }
    }

    fn detect_shell() -> String {
        let shells = vec![
            "/bin/bash",
            "/usr/bin/bash",
            "/run/current-system/sw/bin/bash",
            "/bin/sh",
            "/usr/bin/sh",
            "/run/current-system/sw/bin/sh",
        ];

        for shell in shells {
            if std::path::Path::new(shell).exists() {
                return shell.to_string();
            }
        }

        if let Ok(path) = std::process::Command::new("which").arg("bash").output() {
            let output = String::from_utf8_lossy(&path.stdout);
            let shell = output.trim();
            if !shell.is_empty() && std::path::Path::new(shell).exists() {
                return shell.to_string();
            }
        }

        "/bin/sh".to_string()
    }

    fn send_notification_internal(_job: &Job, _spool_dir: &PathBuf) {
        // Notification system placeholder
    }

    fn check_running(&self) -> usize {
        let registry = self.registry.lock().unwrap();
        registry
            .job_list
            .iter()
            .filter(|j| j.status == JobStatus::Running)
            .count()
    }

    pub fn start(
        &self,
        name: String,
        command: String,
        working_dir: Option<String>,
    ) -> Result<u64, ExecutorError> {
        if self.check_running() >= MAX_CONCURRENT_JOBS {
            return Err(ExecutorError::JobLimitReached);
        }

        let mut registry = self.registry.lock().unwrap();
        let job_id = registry.next_id;

        let output_file = self.spool_dir.join(format!("job_{}.out", job_id));
        let error_file = self.spool_dir.join(format!("job_{}.err", job_id));

        registry.next_id += 1;

        let job = Job {
            id: job_id,
            name,
            command: command.clone(),
            status: JobStatus::Queued,
            pid: None,
            exit_code: None,
            started_at: Utc::now(),
            completed_at: None,
            output_file: output_file.to_string_lossy().to_string(),
            error_file: error_file.to_string_lossy().to_string(),
            working_dir: working_dir.clone().unwrap_or_default(),
        };

        registry.job_list.push(job);
        drop(registry);
        self.save_registry();

        self.spawn_in_background(job_id, command, working_dir);

        Ok(job_id)
    }

    fn spawn_in_background(
        &self,
        job_id: u64,
        command: String,
        working_dir: Option<String>,
    ) {
        let registry = Arc::clone(&self.registry);
        let shell = self.shell_path.clone();
        let spool_dir = self.spool_dir.clone();
        let _launch_dir = self.launch_dir.clone();

        thread::spawn(move || {
            {
                let mut reg = registry.lock().unwrap();
                if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                    job.status = JobStatus::Running;
                }
            }

            let output_file_path = spool_dir.join(format!("job_{}.out", job_id));
            let error_file_path = spool_dir.join(format!("job_{}.err", job_id));

            let output_file = match std::fs::File::create(&output_file_path) {
                Ok(f) => f,
                Err(e) => {
                    let mut reg = registry.lock().unwrap();
                    if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                        job.status = JobStatus::Failed;
                        job.exit_code = Some(-1);
                        job.completed_at = Some(Utc::now());
                    }
                    log_info!("Failed to create output file: {}", e);
                    return;
                }
            };
            let error_file = match std::fs::File::create(&error_file_path) {
                Ok(f) => f,
                Err(e) => {
                    let mut reg = registry.lock().unwrap();
                    if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                        job.status = JobStatus::Failed;
                        job.exit_code = Some(-1);
                        job.completed_at = Some(Utc::now());
                    }
                    log_info!("Failed to create error file: {}", e);
                    return;
                }
            };

            let mut cmd = Command::new(&shell);
            cmd.arg("-c").arg(&command);
            cmd.stdout(output_file);
            cmd.stderr(error_file);

            if let Some(ref dir) = working_dir {
                if !dir.is_empty() {
                    cmd.current_dir(dir);
                }
            }

            let exit_status = cmd.status();

            let mut reg = registry.lock().unwrap();
            if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                match exit_status {
                    Ok(status) => {
                        job.status = if status.success() {
                            JobStatus::Completed
                        } else {
                            JobStatus::Failed
                        };
                        job.exit_code = status.code();
                    }
                    Err(_) => {
                        job.status = JobStatus::Failed;
                        job.exit_code = Some(-1);
                    }
                }
                job.completed_at = Some(Utc::now());
            }
        });
    }

    pub fn kill(&self, id: u64) -> Result<(), ExecutorError> {
        let registry = self.registry.lock().unwrap();
        if let Some(job) = registry.job_list.iter().find(|j| j.id == id) {
            if let Some(pid) = job.pid {
                let _ = kill_process(pid);
            }
        }
        Err(ExecutorError::JobNotFound(id))
    }

    pub fn cancel(&self, id: u64) -> Result<(), ExecutorError> {
        let mut registry = self.registry.lock().unwrap();
        if let Some(job) = registry.job_list.iter_mut().find(|j| j.id == id) {
            if job.status == JobStatus::Queued || job.status == JobStatus::Running {
                job.status = JobStatus::Cancelled;
                if job.status == JobStatus::Running {
                    if let Some(pid) = job.pid {
                        let _ = kill_process(pid);
                    }
                }
                drop(registry);
                self.save_registry();
                return Ok(());
            }
        }
        Err(ExecutorError::JobNotFound(id))
    }

    pub fn list(&self) -> Vec<Job> {
        let registry = self.registry.lock().unwrap();
        registry.job_list.clone()
    }

    pub fn get(&self, id: u64) -> Option<Job> {
        let registry = self.registry.lock().unwrap();
        registry.job_list.iter().find(|j| j.id == id).cloned()
    }

    pub fn dismiss(&self, id: u64) -> Result<(), ExecutorError> {
        let mut registry = self.registry.lock().unwrap();
        if registry.job_list.iter().any(|j| j.id == id) {
            registry.job_list.retain(|j| j.id != id);
            drop(registry);
            self.save_registry();
            Ok(())
        } else {
            Err(ExecutorError::JobNotFound(id))
        }
    }

    pub fn output(&self, id: u64) -> (String, String) {
        let output_file = self.spool_dir.join(format!("job_{}.out", id));
        let error_file = self.spool_dir.join(format!("job_{}.err", id));

        let stdout = std::fs::read_to_string(&output_file).unwrap_or_default();
        let stderr = std::fs::read_to_string(&error_file).unwrap_or_default();

        (stdout, stderr)
    }

    pub fn poll(&self) {
        // Check for completed processes and update their status
        let registry = self.registry.lock().unwrap();
        for job in &registry.job_list {
            if job.status == JobStatus::Running {
                if let Some(pid) = job.pid {
                    let running =
                        match Command::new("ps").arg("-p").arg(pid.to_string()).output() {
                            Ok(output) => {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                output_str.lines().count() > 1
                            }
                            Err(_) => false,
                        };
                    if !running {
                        let mut registry = self.registry.lock().unwrap();
                        if let Some(job) = registry.job_list.iter_mut().find(|j| j.id == job.id) {
                            job.status = JobStatus::Completed;
                            job.exit_code = Some(0);
                            job.completed_at = Some(Utc::now());
                        }
                    }
                }
            }
        }
    }
}

impl JobRegistry {
    fn cleanup_completed(&mut self) {
        let completed: Vec<u64> = self
            .job_list
            .iter()
            .filter(|j| {
                matches!(
                    j.status,
                    JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
                )
            })
            .map(|j| j.id)
            .collect();

        self.job_list.retain(|j| !completed.contains(&j.id));
    }
}

// ============================================================================
// Global accessors
// ============================================================================

static JOB_MANAGER: std::sync::OnceLock<JobManager> = std::sync::OnceLock::new();

pub fn init_job_manager(launch_dir: String) -> &'static JobManager {
    JOB_MANAGER.get_or_init(|| JobManager::new(launch_dir))
}

pub fn get_job_manager() -> &'static JobManager {
    JOB_MANAGER
        .get()
        .expect("JobManager not initialized - call init_job_manager first")
}

pub fn start_spell(
    name: String,
    command: String,
    working_dir: Option<String>,
) -> Result<u64, ExecutorError> {
    get_job_manager().start(name, command, working_dir)
}

pub fn kill_job(id: u64) -> Result<(), ExecutorError> {
    get_job_manager().kill(id)
}

pub fn cancel_job(id: u64) -> Result<(), ExecutorError> {
    get_job_manager().cancel(id)
}

pub fn list_jobs() -> Vec<Job> {
    get_job_manager().list()
}

pub fn get_job(id: u64) -> Option<Job> {
    get_job_manager().get(id)
}

pub fn dismiss_job(id: u64) -> Result<(), ExecutorError> {
    get_job_manager().dismiss(id)
}

pub fn get_job_output(id: u64) -> (String, String) {
    get_job_manager().output(id)
}

pub fn poll_jobs() {
    get_job_manager().poll();
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // === job registry ===

    #[test]
    fn job_status_default_is_queued() {
        assert_eq!(JobStatus::default(), JobStatus::Queued);
    }

    #[test]
    fn job_status_derives_clone() {
        let status = JobStatus::Running;
        let cloned = status.clone();
        assert_eq!(cloned, status);
    }

    #[test]
    fn job_registry_default() {
        let registry = JobRegistry::default();
        assert!(registry.job_list.is_empty());
        assert_eq!(registry.next_id, 1);
    }

    // === executor errors ===

    #[test]
    fn executor_error_display_job_limit() {
        let err = ExecutorError::JobLimitReached;
        assert!(err.to_string().contains("Job limit reached"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn executor_error_display_job_not_found() {
        let err = ExecutorError::JobNotFound(42);
        assert_eq!(err.to_string(), "Job 42 not found");
    }

    #[test]
    fn executor_error_display_shell_not_found() {
        let err = ExecutorError::ShellNotFound;
        assert_eq!(err.to_string(), "No suitable shell found");
    }

    #[test]
    fn executor_error_display_io_error() {
        let err = ExecutorError::IoError("permission denied".to_string());
        assert!(err.to_string().contains("IO error"));
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn executor_error_display_confirmation_required() {
        let err = ExecutorError::ConfirmationRequired("sudo command".to_string());
        assert!(err.to_string().contains("Confirmation required"));
        assert!(err.to_string().contains("sudo command"));
    }

    #[test]
    fn executor_error_implements_error_trait() {
        fn check_error<E: std::error::Error>() {}
        check_error::<ExecutorError>();
    }
}
