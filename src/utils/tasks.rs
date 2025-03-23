//! Task management module
//!
//! This module provides a task management system for tracking background tasks

use crate::ai::types::{ProgressStats, TaskStatus};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use uuid::Uuid;

/// Unique identifier for a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(Uuid);

impl TaskId {
    /// Create a new random task ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get a short string representation
    pub fn short(&self) -> String {
        self.0
            .to_string()
            .split('-')
            .next()
            .unwrap_or_default()
            .to_string()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

/// Type of task
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    /// AI generation task
    AIGeneration,
    /// Bash command execution
    BashCommand,
    /// File operation (read/write)
    FileOperation,
    /// Network request
    NetworkRequest,
    /// Other generic task
    Other,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::AIGeneration => write!(f, "AI Generation"),
            TaskType::BashCommand => write!(f, "Bash Command"),
            TaskType::FileOperation => write!(f, "File Operation"),
            TaskType::NetworkRequest => write!(f, "Network Request"),
            TaskType::Other => write!(f, "Task"),
        }
    }
}

/// A background task with metadata
#[derive(Debug, Clone)]
pub struct Task {
    /// Unique ID for the task
    pub id: TaskId,
    /// Human-readable name
    pub name: String,
    /// Type of task
    pub task_type: TaskType,
    /// Current status
    pub status: TaskStatus,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Start time (when started running)
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Progress statistics
    pub progress: Option<ProgressStats>,
    /// Task description (optional)
    pub description: Option<String>,
}

impl Task {
    /// Create a new task
    pub fn new(name: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: TaskId::new(),
            name: name.into(),
            task_type,
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: None,
            description: None,
        }
    }

    /// Mark the task as running
    pub fn mark_running(&mut self) {
        self.status = TaskStatus::Running;
        self.started_at = Some(Utc::now());
    }

    /// Mark the task as completed
    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
        if let Some(progress) = &mut self.progress {
            progress.complete();
        }
    }

    /// Mark the task as failed
    pub fn mark_failed(&mut self) {
        self.status = TaskStatus::Failed;
        self.completed_at = Some(Utc::now());
    }

    /// Mark the task as cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
    }

    /// Set the task's progress stats
    pub fn set_progress(&mut self, progress: ProgressStats) {
        self.progress = Some(progress);
    }

    /// Update the task's progress
    pub fn update_progress(&mut self, tokens_generated: usize) {
        if let Some(progress) = &mut self.progress {
            progress.update(tokens_generated);
        } else {
            let mut progress = ProgressStats::new();
            progress.update(tokens_generated);
            self.progress = Some(progress);
        }
    }

    /// Get the task's duration in seconds
    pub fn duration_seconds(&self) -> f64 {
        let end_time = self.completed_at.unwrap_or_else(Utc::now);
        let start_time = self.started_at.unwrap_or(self.created_at);
        (end_time - start_time).num_milliseconds() as f64 / 1000.0
    }

    /// Get a formatted string with the task's duration
    pub fn format_duration(&self) -> String {
        let seconds = self.duration_seconds();
        if seconds < 1.0 {
            format!("{:.2}s", seconds)
        } else if seconds < 60.0 {
            format!("{:.1}s", seconds)
        } else if seconds < 3600.0 {
            let minutes = (seconds / 60.0).floor();
            let remaining_seconds = seconds % 60.0;
            format!("{}m {:.0}s", minutes, remaining_seconds)
        } else {
            let hours = (seconds / 3600.0).floor();
            let remaining_minutes = ((seconds % 3600.0) / 60.0).floor();
            format!("{}h {}m", hours, remaining_minutes)
        }
    }
}

/// Manager for background tasks
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    tx: broadcast::Sender<TaskId>,
    // Store response channels for tasks that return content
    response_channels: Arc<Mutex<HashMap<TaskId, mpsc::Receiver<Option<String>>>>>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    /// Create a new task manager
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            tx,
            response_channels: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Store a response channel for a task
    pub fn set_response_channel(&self, id: TaskId, rx: mpsc::Receiver<Option<String>>) {
        let mut channels = self.response_channels.lock().unwrap();
        channels.insert(id, rx);
    }
    
    /// Get a response channel for a task
    pub fn take_response_channel(&self, id: TaskId) -> Option<mpsc::Receiver<Option<String>>> {
        let mut channels = self.response_channels.lock().unwrap();
        channels.remove(&id)
    }

    /// Create and register a new task
    pub fn create_task(&self, name: impl Into<String>, task_type: TaskType) -> TaskId {
        let task = Task::new(name, task_type);
        let id = task.id;

        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(id, task);

        // Notify listeners
        let _ = self.tx.send(id);

        id
    }

    /// Get a task by ID
    pub fn get_task(&self, id: TaskId) -> Option<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).cloned()
    }

    /// Update a task's status
    pub fn update_task_status(&self, id: TaskId, status: TaskStatus) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            match status {
                TaskStatus::Pending => {
                    // No state change for pending
                }
                TaskStatus::Running => {
                    task.mark_running();
                }
                TaskStatus::Completed => {
                    task.mark_completed();
                }
                TaskStatus::Failed => {
                    task.mark_failed();
                }
                TaskStatus::Cancelled => {
                    task.mark_cancelled();
                }
            }

            // Notify listeners with broadcast
            let _ = self.tx.send(id);

            true
        } else {
            false
        }
    }

    /// Update a task's progress
    pub fn update_task_progress(&self, id: TaskId, tokens_generated: usize) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            task.update_progress(tokens_generated);

            // Notify listeners with broadcast
            let _ = self.tx.send(id);

            true
        } else {
            false
        }
    }

    /// Cancel a task by ID
    pub fn cancel_task(&self, id: TaskId) -> bool {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            task.mark_cancelled();

            // Notify listeners with broadcast
            let _ = self.tx.send(id);

            true
        } else {
            false
        }
    }

    /// Get a list of all active tasks
    pub fn active_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks
            .values()
            .filter(|task| task.status == TaskStatus::Running || task.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }

    /// Get a list of all tasks
    pub fn all_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        tasks.values().cloned().collect()
    }

    /// Get a list of recently completed tasks (last 10 minutes)
    pub fn recent_tasks(&self) -> Vec<Task> {
        let tasks = self.tasks.lock().unwrap();
        let now = Utc::now();
        tasks
            .values()
            .filter(|task| {
                if let Some(completed_at) = task.completed_at {
                    (now - completed_at).num_minutes() < 10
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Get the task update channel
    pub fn get_update_receiver(&self) -> mpsc::Receiver<TaskId> {
        let (tx, rx) = mpsc::channel(100);

        // Create a new broadcast receiver from the main channel
        let mut broadcast_rx = self.tx.subscribe();
        
        // Spawn a task to forward broadcast messages to the mpsc channel
        tokio::spawn(async move {
            while let Ok(task_id) = broadcast_rx.recv().await {
                if tx.send(task_id).await.is_err() {
                    // Receiver was dropped, stop forwarding
                    break;
                }
            }
        });

        rx
    }

    /// Clean up old tasks to prevent memory leaks
    pub fn cleanup_old_tasks(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        let now = Utc::now();

        // Remove completed tasks older than 30 minutes
        tasks.retain(|_, task| {
            if let Some(completed_at) = task.completed_at {
                (now - completed_at).num_minutes() < 30
            } else {
                true
            }
        });
    }
}