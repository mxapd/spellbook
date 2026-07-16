use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const MAX_CONCURRENT_JOBS: usize = 10;
const SPELLBOOK_DIR: &str = ".spellbook";
const JOBS_FILE: &str = "jobs.toml";
const POLL_INTERVAL: Duration = Duration::from_secs(1);

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
            ExecutorError::JobLimitReached => write!(
                f,
                "Job limit reached (max {} concurrent jobs)",
                MAX_CONCURRENT_JOBS
            ),
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

    fn spool_dir(&self) -> &PathBuf {
        &self.spool_dir
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

    fn shell_escape(cmd: &str) -> String {
        cmd.replace("'", "'\\''")
    }

    pub fn running_count(&self) -> usize {
        let registry = self.registry.lock().unwrap();
        registry
            .job_list
            .iter()
            .filter(|j| j.status == JobStatus::Running || j.status == JobStatus::Queued)
            .count()
    }

    pub fn start(
        &self,
        name: String,
        command: String,
        working_dir: Option<String>,
    ) -> Result<u64, ExecutorError> {
        let running = self.running_count();
        if running >= MAX_CONCURRENT_JOBS {
            return Err(ExecutorError::JobLimitReached);
        }

        let mut registry = self.registry.lock().unwrap();
        let id = registry.next_id;
        registry.next_id += 1;

        let output_file = format!("job_{:03}.out", id);
        let error_file = format!("job_{:03}.err", id);
        let output_path = self.spool_dir.join(&output_file);
        let error_path = self.spool_dir.join(&error_file);

        let job = Job {
            id,
            name: name.clone(),
            command: command.clone(),
            status: JobStatus::Running,
            pid: None,
            exit_code: None,
            started_at: Utc::now(),
            completed_at: None,
            output_file,
            error_file,
            working_dir: working_dir.clone().unwrap_or_default(),
        };

        registry.job_list.push(job);
        drop(registry);

        self.save_registry();
        self.spawn_detached(id, command, output_path, error_path, working_dir);

        Ok(id)
    }

    fn _spawn_detached_with_sudo(
        &self,
        job_id: u64,
        command: String,
        password: String,
        _output_path: PathBuf,
        _error_path: PathBuf,
    ) {
        let spool_dir = self.spool_dir.clone();
        let shell_path = self.shell_path.clone();
        let registry = Arc::clone(&self.registry);

        thread::spawn(move || {
            let output_file = spool_dir.join(&format!("job_{:03}.out", job_id));
            let error_file = spool_dir.join(&format!("job_{:03}.err", job_id));

            let _ = fs::File::create(&output_file);
            let _ = fs::File::create(&error_file);

            let sudo_cmd = format!(
                "nohup sudo -S {} -c '{}' > {} 2> {} < /dev/null & disown; echo $!",
                shell_path,
                Self::shell_escape(&command),
                output_file.display(),
                error_file.display()
            );

            let result = Command::new("bash")
                .arg("-c")
                .arg(&sudo_cmd)
                .env("SHELL", &shell_path)
                .env(
                    "HOME",
                    std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
                )
                .env("SUDO_ASKPASS", "/bin/cat")
                .env("SUDO_PASSWORD", password)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output();

            match result {
                Ok(output) => {
                    let stderr_str = String::from_utf8_lossy(&output.stderr);
                    if stderr_str.contains("incorrect password")
                        || stderr_str.contains("sudo: 3 incorrect password attempts")
                    {
                        let mut reg = registry.lock().unwrap();
                        if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                            job.status = JobStatus::Failed;
                            job.exit_code = Some(-1);
                            job.completed_at = Some(Utc::now());
                        }
                        return;
                    }

                    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        let mut reg = registry.lock().unwrap();
                        if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                            job.pid = Some(pid);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to spawn job {}: {}", job_id, e);
                    let mut reg = registry.lock().unwrap();
                    if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                        job.status = JobStatus::Failed;
                        job.exit_code = Some(-1);
                        job.completed_at = Some(Utc::now());
                    }
                }
            }
        });
    }

    fn spawn_detached(
        &self,
        job_id: u64,
        command: String,
        _output_path: PathBuf,
        _error_path: PathBuf,
        working_dir: Option<String>,
    ) {
        let spool_dir = self.spool_dir.clone();
        let shell_path = self.shell_path.clone();
        let registry = Arc::clone(&self.registry);
        let launch_dir = self.launch_dir.clone();

        let default_dir = if std::path::Path::new(&launch_dir).exists() {
            launch_dir.clone()
        } else {
            std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
        };

        let work_dir = working_dir
            .filter(|d| !d.is_empty() && std::path::Path::new(d).exists())
            .unwrap_or(default_dir);

        thread::spawn(move || {
            let output_file = spool_dir.join(&format!("job_{:03}.out", job_id));
            let error_file = spool_dir.join(&format!("job_{:03}.err", job_id));

            let _ = fs::File::create(&output_file);
            let _ = fs::File::create(&error_file);

            let cmd = format!(
                "cd '{}' && nohup {} -c '{}' > {} 2> {} < /dev/null & disown; echo $!",
                work_dir,
                shell_path,
                Self::shell_escape(&command),
                output_file.display(),
                error_file.display()
            );
            let result = Command::new("bash").arg("-c").arg(&cmd).output();

            match result {
                Ok(output) => {
                    let pid_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        let mut reg = registry.lock().unwrap();
                        if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                            job.pid = Some(pid);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to spawn job {}: {}", job_id, e);
                    let mut reg = registry.lock().unwrap();
                    if let Some(job) = reg.job_list.iter_mut().find(|j| j.id == job_id) {
                        job.status = JobStatus::Failed;
                        job.exit_code = Some(-1);
                        job.completed_at = Some(Utc::now());
                    }
                }
            }
        });
    }

    pub fn kill(&self, id: u64) -> Result<(), ExecutorError> {
        let mut registry = self.registry.lock().unwrap();
        if let Some(job) = registry.job_list.iter_mut().find(|j| j.id == id) {
            if let Some(pid) = job.pid {
                let _ = Command::new("kill").arg("-9").arg(pid.to_string()).output();
            }
            job.status = JobStatus::Cancelled;
            job.completed_at = Some(Utc::now());
            drop(registry);
            self.save_registry();
            Ok(())
        } else {
            Err(ExecutorError::JobNotFound(id))
        }
    }

    pub fn cancel(&self, id: u64) -> Result<(), ExecutorError> {
        let mut registry = self.registry.lock().unwrap();
        if let Some(job) = registry.job_list.iter_mut().find(|j| j.id == id) {
            if job.status == JobStatus::Queued || job.status == JobStatus::Running {
                if let Some(pid) = job.pid {
                    let _ = Command::new("kill").arg(pid.to_string()).output();
                }
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(Utc::now());
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
        if let Some(pos) = registry.job_list.iter().position(|j| j.id == id) {
            let job = registry.job_list.remove(pos);
            drop(registry);
            self.save_registry();

            let out_path = self.spool_dir.join(&job.output_file);
            let err_path = self.spool_dir.join(&job.error_file);
            let _ = fs::remove_file(out_path);
            let _ = fs::remove_file(err_path);

            Ok(())
        } else {
            Err(ExecutorError::JobNotFound(id))
        }
    }

    pub fn output(&self, id: u64) -> (String, String) {
        let registry = self.registry.lock().unwrap();
        if let Some(job) = registry.job_list.iter().find(|j| j.id == id) {
            let out_path = self.spool_dir.join(&job.output_file);
            let err_path = self.spool_dir.join(&job.error_file);
            drop(registry);

            let stdout = fs::read_to_string(&out_path).unwrap_or_default();
            let stderr = fs::read_to_string(&err_path).unwrap_or_default();
            (stdout, stderr)
        } else {
            (String::new(), String::new())
        }
    }

    pub fn poll(&self) {
        let mut registry = self.registry.lock().unwrap();

        for job in registry.job_list.iter_mut() {
            if job.status == JobStatus::Running {
                if let Some(pid) = job.pid {
                    match Command::new("ps").arg("-p").arg(pid.to_string()).output() {
                        Ok(output) => {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            if output_str.lines().count() <= 1 {
                                let status_file =
                                    self.spool_dir.join(format!("job_{:03}.status", job.id));
                                let exit_code = if status_file.exists() {
                                    fs::read_to_string(&status_file)
                                        .ok()
                                        .and_then(|s| s.trim().parse::<i32>().ok())
                                } else {
                                    None
                                };

                                job.status = if exit_code == Some(0) {
                                    JobStatus::Completed
                                } else {
                                    JobStatus::Failed
                                };
                                job.exit_code = exit_code.or(Some(1));
                                job.completed_at = Some(Utc::now());

                                Self::send_notification(&job);
                            }
                        }
                        Err(_) => {
                            job.status = JobStatus::Failed;
                            job.exit_code = Some(-1);
                            job.completed_at = Some(Utc::now());
                            Self::send_notification(&job);
                        }
                    }
                }
            }
        }

        drop(registry);
        self.save_registry();
    }

    fn send_notification(job: &Job) {
        Self::send_notification_internal(job, &PathBuf::new());
    }

    fn send_notification_internal(job: &Job, _spool_dir: &PathBuf) {
        let (icon, urgency) = match job.status {
            JobStatus::Completed => ("dialog-information", "normal"),
            JobStatus::Failed => ("dialog-error", "critical"),
            JobStatus::Cancelled => ("dialog-warning", "normal"),
            _ => return,
        };

        let message = match job.status {
            JobStatus::Completed => format!("{} completed", job.name),
            JobStatus::Failed => {
                format!("{} failed (exit {})", job.name, job.exit_code.unwrap_or(-1))
            }
            JobStatus::Cancelled => format!("{} cancelled", job.name),
            _ => return,
        };

        let _ = Command::new("notify-send")
            .arg("-u")
            .arg(urgency)
            .arg("--icon")
            .arg(icon)
            .arg("Spellbook")
            .arg(&message)
            .spawn();
    }

    pub fn clear_completed(&self) {
        let mut registry = self.registry.lock().unwrap();
        registry.cleanup_completed();
        drop(registry);
        self.save_registry();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_default_is_queued() {
        assert_eq!(JobStatus::default(), JobStatus::Queued);
    }

    #[test]
    fn test_job_status_derives_clone() {
        let status = JobStatus::Running;
        let cloned = status.clone();
        assert_eq!(cloned, status);
    }

    #[test]
    fn test_job_registry_default() {
        let registry = JobRegistry::default();
        assert!(registry.job_list.is_empty());
        assert_eq!(registry.next_id, 1);
    }

    #[test]
    fn test_executor_error_display_job_limit() {
        let err = ExecutorError::JobLimitReached;
        assert!(err.to_string().contains("Job limit reached"));
        assert!(err.to_string().contains("10"));
    }

    #[test]
    fn test_executor_error_display_job_not_found() {
        let err = ExecutorError::JobNotFound(42);
        assert_eq!(err.to_string(), "Job 42 not found");
    }

    #[test]
    fn test_executor_error_display_shell_not_found() {
        let err = ExecutorError::ShellNotFound;
        assert_eq!(err.to_string(), "No suitable shell found");
    }

    #[test]
    fn test_executor_error_display_io_error() {
        let err = ExecutorError::IoError("permission denied".to_string());
        assert!(err.to_string().contains("IO error"));
        assert!(err.to_string().contains("permission denied"));
    }

    #[test]
    fn test_executor_error_display_confirmation_required() {
        let err = ExecutorError::ConfirmationRequired("sudo command".to_string());
        assert!(err.to_string().contains("Confirmation required"));
        assert!(err.to_string().contains("sudo command"));
    }

    #[test]
    fn test_executor_error_implements_error_trait() {
        fn check_error<E: std::error::Error>() {}
        check_error::<ExecutorError>();
    }

    #[test]
    fn test_execute_sync_captures_stdout() {
        let result = execute_sync("echo hello");
        assert_eq!(result.stdout.trim(), "hello");
        assert!(result.stderr.is_empty());
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_execute_sync_captures_stderr() {
        let result = execute_sync("bash -c 'echo error >&2'");
        assert_eq!(result.stderr.trim(), "error");
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_execute_sync_exit_code_success() {
        let result = execute_sync("true");
        assert_eq!(result.exit_code, Some(0));
    }

    #[test]
    fn test_execute_sync_exit_code_failure() {
        let result = execute_sync("false");
        assert_eq!(result.exit_code, Some(1));
    }

    #[test]
    fn test_execute_sync_handles_invalid_command() {
        let result = execute_sync("nonexistent_command_12345");
        assert!(result.exit_code.is_some());
        assert!(result.exit_code.unwrap() != 0 || !result.stderr.is_empty());
    }

    #[test]
    fn test_execute_sync_multiline_output() {
        let result = execute_sync("echo line1 && echo line2 && echo line3");
        let lines: Vec<&str> = result.stdout.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    #[test]
    fn test_execute_sync_special_characters() {
        let result = execute_sync(r#"echo "hello world" '#'"#);
        assert!(result.stdout.contains("hello world"));
    }
}

pub struct SyncExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub pid: u32,
}

pub fn execute_sync(command: &str) -> SyncExecutionResult {
    let output = Command::new("bash").arg("-c").arg(command).output();

    match output {
        Ok(out) => SyncExecutionResult {
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            exit_code: out.status.code(),
            pid: 0,
        },
        Err(e) => SyncExecutionResult {
            stdout: String::new(),
            stderr: format!("Failed to execute: {}", e),
            exit_code: Some(127),
            pid: 0,
        },
    }
}

pub fn execute_sync_spawn(command: &str) -> std::io::Result<(SyncExecutionResult, u32)> {
    use std::process::{Command, Stdio};

    let child = Command::new("bash")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();

    let output = child.wait_with_output()?;

    let result = SyncExecutionResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        exit_code: output.status.code(),
        pid,
    };

    Ok((result, pid))
}

pub fn kill_process(pid: u32) -> Result<(), std::io::Error> {
    use std::process::Command;
    Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .output()?;
    Ok(())
}

fn detect_shell_static() -> String {
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

#[derive(Debug, Clone)]
pub struct Placeholder {
    pub original: String,
    pub name: String,
    pub display_name: String,
}

pub fn detect_placeholders(command: &str) -> Vec<Placeholder> {
    use regex::Regex;

    let re = Regex::new(r"<([^>]+)>").unwrap();
    re.captures_iter(command)
        .map(|cap| {
            let name = cap[1].to_string();
            let display_name = match name.as_str() {
                "pid" => "Process ID",
                "port" => "Port",
                "package" => "Package",
                "file" => "File",
                "directory" | "dir" => "Directory",
                "service" | "svc" => "Service",
                "message" | "msg" => "Message",
                "image" | "img" => "Image",
                "container" => "Container",
                "signal" => "Signal",
                "user" => "User",
                "host" => "Host",
                "name" => "Name",
                "tag" => "Tag",
                _ => &name,
            }
            .to_string();

            Placeholder {
                original: cap[0].to_string(),
                name: name.clone(),
                display_name,
            }
        })
        .collect()
}

pub fn substitute_placeholders(command: &str, values: &[(String, String)]) -> String {
    let mut result = command.to_string();
    for (name, value) in values {
        result = result.replace(&format!("<{}>", name), value);
    }
    result
}

pub fn validate_password() -> bool {
    let shell = detect_shell_static();

    let result = Command::new("sudo")
        .arg("-v")
        .env("SHELL", &shell)
        .env(
            "HOME",
            std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
        )
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

pub struct StreamOutput {
    pub line: String,
    pub is_stderr: bool,
}

pub fn stream_command(
    command: &str,
    working_dir: Option<&str>,
    launch_dir: &str,
) -> std::io::Result<(
    u32,
    thread::JoinHandle<()>,
    std::sync::mpsc::Receiver<StreamOutput>,
)> {
    let shell = detect_shell_static();

    let mut cmd = Command::new(&shell);
    cmd.arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let default_dir = if std::path::Path::new(launch_dir).exists() {
        launch_dir.to_string()
    } else {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    };

    let work_dir = working_dir
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string())
        .unwrap_or(default_dir);

    let dir = PathBuf::from(work_dir);
    if dir.exists() && dir.is_dir() {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let pid = child.id();

    let (tx, rx) = std::sync::mpsc::channel::<StreamOutput>();
    let tx_for_stream = tx.clone();

    let handle = thread::spawn(move || {
        let mut reader_threads = vec![];

        if let Some(stdout) = stdout {
            let tx_clone = tx_for_stream.clone();
            let t = thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_clone.send(StreamOutput {
                        line,
                        is_stderr: false,
                    });
                }
            });
            reader_threads.push(t);
        }

        if let Some(stderr) = stderr {
            let tx_clone = tx_for_stream.clone();
            let t = thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = tx_clone.send(StreamOutput {
                        line,
                        is_stderr: true,
                    });
                }
            });
            reader_threads.push(t);
        }

        for t in reader_threads {
            let _ = t.join();
        }

        let _ = tx_for_stream.send(StreamOutput {
            line: String::new(),
            is_stderr: false,
        });
    });

    Ok((pid, handle, rx))
}

#[cfg(unix)]
pub fn exec_simple(command: &str, working_dir: Option<&str>, launch_dir: &str) -> ! {
    use std::env;
    use std::io::Write;

    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let default_dir = if std::path::Path::new(launch_dir).exists() {
        launch_dir.to_string()
    } else {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    };

    let work_dir = working_dir
        .filter(|d| !d.is_empty())
        .map(|d| d.to_string())
        .unwrap_or(default_dir);

    let _ = std::env::set_current_dir(work_dir);

    let program = std::path::Path::new(&shell)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("sh");

    // Leave alternate screen and reset SGR before exec
    // \x1b[?1049l = Leave alternate screen
    // \x1b[0m = Reset SGR (colors, bold, etc.)
    let cleanup = "\x1b[?1049l\x1b[0m";
    let _ = std::io::stdout().write_all(cleanup.as_bytes());
    let _ = std::io::stdout().flush();

    // execvp only returns on error, and the error type is #[must_use]
    let _error = exec::execvp(&shell, &[program, "-c", command]);
    std::process::exit(1)
}

#[cfg(not(unix))]
pub fn exec_simple(command: &str, working_dir: Option<&str>, launch_dir: &str) -> i32 {
    use std::env;

    let shell = env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    let default_dir = if std::path::Path::new(launch_dir).exists() {
        launch_dir
    } else {
        env::var("HOME").unwrap_or_else(|_| "/").as_str()
    };

    let mut cmd = Command::new(&shell);
    cmd.arg("-c").arg(command);

    let work_dir = working_dir
        .filter(|d| !d.is_empty() && std::path::Path::new(d).exists())
        .unwrap_or(default_dir);

    cmd.current_dir(work_dir);
    cmd.env("HOME", env::var("HOME").unwrap_or_else(|_| "/".to_string()));

    match cmd.status() {
        Ok(status) => status.code().unwrap_or(0),
        Err(e) => {
            eprintln!("Failed to execute: {}", e);
            1
        }
    }
}
