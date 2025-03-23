//! Main application state and event handling

use crate::handlers::{bash, command};
use anyhow::Result;
use chrono::{DateTime, Local};
use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::VecDeque;
use std::env;
use std::io;
use std::path::PathBuf;
use std::sync::{mpsc, Arc};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::event::Event;
use crate::handlers::CommandMode;
use crate::tui::Tui;
use crate::ui;
use crate::utils::{Colors, TaskManager};

mod ai_handler;
use ai_handler::AIHandler;

pub type AppResult<T> = Result<T>;

// Session statistics
pub struct SessionStats {
    pub start_time: DateTime<Local>,
    pub command_count: usize,
    pub ai_count: usize,
    pub bash_count: usize,
    pub cost: f64,
    pub total_tokens: usize,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            start_time: Local::now(),
            command_count: 0,
            ai_count: 0,
            bash_count: 0,
            cost: 0.0,
            total_tokens: 0,
            prompt_tokens: 0,
            completion_tokens: 0,
        }
    }
}

// Command history
pub struct History {
    pub commands: VecDeque<String>,
    pub position: usize,
    pub max_size: usize,
}

impl Default for History {
    fn default() -> Self {
        Self {
            commands: VecDeque::with_capacity(100),
            position: 0,
            max_size: 100,
        }
    }
}

impl History {
    pub fn add(&mut self, command: String) {
        if !command.trim().is_empty() {
            // Keep history size within limits
            if self.commands.len() >= self.max_size {
                self.commands.pop_front();
            }
            self.commands.push_back(command);
            self.position = self.commands.len();
        }
    }
}

// Main application state
pub struct App {
    pub running: bool,
    pub input: String,
    pub cursor_position: usize,      // Track cursor position in input
    pub cursor_visible: bool,        // Toggle for cursor blinking
    pub last_cursor_toggle: Instant, // Time of last cursor blink
    pub output: String,
    pub history: History,
    pub current_dir: PathBuf,
    pub colors: Colors,
    pub stats: SessionStats,
    pub current_mode: CommandMode,
    pub scroll_offset: u16,
    pub is_selecting_text: bool,
    pub selection_start: usize,
    pub selection_end: usize,
    pub output_lines: Vec<String>,
    pub show_context_menu: bool,
    pub context_menu_x: u16,
    pub context_menu_y: u16,
    pub mouse_drag_start_x: u16,
    pub mouse_drag_start_y: u16,
    pub mouse_drag_ongoing: bool,
    pub output_area_height: u16,    // To track output area dimensions
    pub last_click_time: Instant,   // For double click detection
    pub last_click_pos: (u16, u16), // For double click detection
    pub native_selection_mode: bool,
    pub is_scrolling: bool, // Track when scrolling is in progress
    pub ai_handler: AIHandler,
    pub spinner_rx: Option<mpsc::Receiver<(String, usize)>>, // Receiver for spinner updates
    pub abort_requested: Arc<AtomicBool>, // Atomic flag to indicate if abort was requested
    pub global_abort: Option<Arc<AtomicBool>>, // Global atomic abort flag
    pub ui_notifier: Option<tokio::sync::mpsc::Sender<()>>, // Channel to request UI updates
    pub background_tasks: Vec<tokio::task::JoinHandle<()>>, // Track background tasks
    pub task_manager: TaskManager, // Manager for background tasks
    pub show_tasks_popup: bool, // Whether to show the tasks popup
    pub last_cleanup_time: Option<Instant>, // Last time task cleanup was performed
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input: String::new(),
            cursor_position: 0,   // Initialize cursor at beginning of input
            cursor_visible: true, // Start with visible cursor
            last_cursor_toggle: Instant::now(), // Initialize cursor blink timer
            output: String::new(),
            history: History::default(),
            current_dir: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            colors: Colors::default(),
            stats: SessionStats::default(),
            current_mode: CommandMode::AI,
            scroll_offset: 0,
            is_selecting_text: false,
            selection_start: 0,
            selection_end: 0,
            output_lines: Vec::new(),
            show_context_menu: false,
            context_menu_x: 0,
            context_menu_y: 0,
            mouse_drag_start_x: 0,
            mouse_drag_start_y: 0,
            mouse_drag_ongoing: false,
            output_area_height: 0,
            last_click_time: Instant::now(),
            last_click_pos: (0, 0),
            native_selection_mode: true,
            is_scrolling: false, // Initialize scrolling state
            ai_handler: AIHandler::new(),
            spinner_rx: None, // Initialize spinner receiver as None
            abort_requested: Arc::new(AtomicBool::new(false)), // Initialize abort flag as false
            global_abort: None, // Initialize global abort flag as None,
            ui_notifier: None, // Will be set after construction
            background_tasks: Vec::new(), // Start with no background tasks
            task_manager: TaskManager::new(), // Initialize task manager
            show_tasks_popup: false, // Don't show tasks popup by default
            last_cleanup_time: None, // Initialize cleanup timer to None
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_global_abort(&mut self, abort_flag: Arc<AtomicBool>) {
        self.global_abort = Some(abort_flag);
    }
    
    pub fn is_abort_requested(&self) -> bool {
        self.abort_requested.load(std::sync::atomic::Ordering::SeqCst) || 
        self.global_abort.as_ref().is_some_and(|flag| flag.load(std::sync::atomic::Ordering::SeqCst))
    }

    pub fn add_output(&mut self, text: String) {
        // Process the text based on whether it ends with a newline
        let text = if text.ends_with('\n') {
            text
        } else {
            // Only add a single newline if needed
            text + "\n"
        };
        
        // Add the text to the output string
        self.output.push_str(&text);

        // Update output_lines for text selection and copying
        // Split by newline to get individual lines
        let new_lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        
        // Add each line to our output_lines vector
        self.output_lines.extend(new_lines);
        
        // Only add an empty line if absolutely needed for empty lines
        if text.ends_with('\n') && text.trim().is_empty() {
            self.output_lines.push(String::new());
        }
    }

    pub fn format_timestamp(&self) -> String {
        Local::now().format("%H:%M").to_string()
    }

    pub fn detect_mode(&self, command: &str) -> (CommandMode, String) {
        let command = command.trim();

        if let Some(stripped) = command.strip_prefix('!') {
            (CommandMode::Bash, stripped.trim().to_string())
        } else if let Some(stripped) = command.strip_prefix('/') {
            (CommandMode::Command, stripped.trim().to_string())
        } else {
            (CommandMode::AI, command.to_string())
        }
    }

    pub async fn execute_command(&mut self, command: String, tui: &mut Tui) {
        // Clean up any excessive newlines at the end of the current output
        while self.output.ends_with("\n\n") {
            self.output.pop();
        }
        
        // Add command to history
        self.history.add(command.clone());

        // Detect mode and get processed command
        let (mode, cmd) = self.detect_mode(&command);

        // Add a separator between commands (more compact)
        self.add_output("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n".to_string());

        // Always format and display the command first, before any processing happens
        match mode {
            CommandMode::Bash => self.add_output(format!("$ {}", command)),
            CommandMode::Command => self.add_output(format!("/ {}", command)),
            CommandMode::AI => self.add_output(format!("❯ {}", command)),
        };

        // Force immediate UI refresh to show the command right away
        if let Err(e) = tui.immediate_refresh(|f| {
            ui::render(f, self);
        }) {
            eprintln!("Failed to refresh UI: {}", e);
        }

        match mode {
            CommandMode::Bash => {
                // Add a newline for better readability
                self.add_output("\n".to_string());

                // Now execute the command
                let result =
                    bash::handle_bash_command(&cmd).unwrap_or_else(|e| format!("Error: {}", e));
                self.add_output(result);
                self.stats.bash_count += 1;
            }
            CommandMode::Command => {
                // Add a newline for better readability
                self.add_output("\n".to_string());

                // Handle special cases
                if &cmd == "clear" {
                    self.output = "🚀 Output cleared\n".to_string();
                    self.output_lines.clear();
                    return;
                } else if &cmd == "cost" {
                    // Use our app's internal stats for cost reporting
                    let cost_info = self.get_session_cost_info();
                    self.add_output(cost_info);
                    self.stats.command_count += 1;
                    return;
                }

                // Execute command with robust error handling
                match command::CommandHandler::handle_command(&cmd) {
                    Ok(result) => {
                        // Special handling for commands that might modify the AI client
                        if cmd.starts_with("provider") || cmd.starts_with("model") {
                            self.add_output(result.clone());

                            // Attempt to update the AI client - log errors but don't crash
                            if let Err(e) = self.ai_handler.update_client() {
                                self.add_output(format!(
                                    "⚠️ Warning: Could not update AI client: {}\n",
                                    e
                                ));
                            } else {
                                self.add_output("✅ AI client updated successfully\n".to_string());
                            }
                        } else {
                            self.add_output(result);
                        }
                    }
                    Err(e) => {
                        self.add_output(format!("Error: {}", e));
                    }
                }
                self.stats.command_count += 1;
            }
            CommandMode::AI => {
                // Add a minimal spinner indicator with no extra space
                self.add_output("".to_string());

                // Immediately refresh UI
                if let Err(e) = tui.immediate_refresh(|f| {
                    ui::render(f, self);
                }) {
                    eprintln!("Failed to refresh UI: {}", e);
                }

                // Create a new channel for spinner animation
                let (tx, rx) = mpsc::channel();
                self.spinner_rx = Some(rx);
                
                // Determine the line index for the spinner (the last line in output_lines)
                let spinner_line_index = self.output_lines.len() - 1;
                
                // Save a reference to our global abort flag for the spinner task
                let global_abort_clone = self.global_abort.clone();
                
                // Spawn spinner task with proper line index and abort checking
                let spinner_task = tokio::spawn(async move {
                    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                    let mut frame = 0;
                    
                    loop {
                        // Check if we should abort
                        let should_abort = global_abort_clone
                            .as_ref()
                            .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::SeqCst));
                            
                        if should_abort {
                            break;
                        }
                        
                        // Send both the spinner frame and its line index
                        if tx.send((spinner_frames[frame].to_string(), spinner_line_index)).is_err() {
                            break;
                        }
                        
                        frame = (frame + 1) % spinner_frames.len();
                        tokio::time::sleep(Duration::from_millis(80)).await;
                    }
                });

                // Reset abort flags before starting
                self.abort_requested.store(false, std::sync::atomic::Ordering::SeqCst);
                if let Some(global_abort) = &self.global_abort {
                    global_abort.store(false, std::sync::atomic::Ordering::SeqCst);
                }

                // Get shared references to what we need for the task 
                let abort_flag = self.abort_requested.clone();
                let global_abort_clone = self.global_abort.clone();
                let cmd_clone = cmd.clone();
                let ai_handler_clone = self.ai_handler.clone();
                let ui_tx = self.ui_notifier.clone();
                
                // Create a task in the task manager
                let task_id = self.task_manager.create_task(
                    format!("AI: {}", cmd.chars().take(30).collect::<String>()),
                    crate::utils::tasks::TaskType::AIGeneration
                );
                
                // Mark task as running
                self.task_manager.update_task_status(task_id, crate::ai::types::TaskStatus::Running);
                
                // Create a task progress update channel
                let task_manager = self.task_manager.clone();
                
                // Use a truly concurrent approach by spawning the AI generation in a separate task
                let ai_task = tokio::spawn(async move {
                    // We'll use the atomic abort flag for thread-safe cancellation
                    
                    // Run the AI generation with a timeout to prevent hanging
                    let result = tokio::time::timeout(
                        std::time::Duration::from_secs(120), // Increase timeout for larger models
                        ai_handler_clone.generate(&cmd_clone, abort_flag, global_abort_clone)
                    ).await;
                    
                    // Update task status based on result
                    match &result {
                        Ok(Ok(response)) => {
                            // If the response has progress stats, update the task
                            if let Some(progress) = &response.progress {
                                task_manager
                                    .update_task_progress(task_id, progress.tokens_generated);
                            }
                            task_manager.update_task_status(
                                task_id,
                                crate::ai::types::TaskStatus::Completed,
                            );
                        }
                        Ok(Err(e)) => {
                            if let crate::ai::AIError::Cancelled(_) = e {
                                task_manager.update_task_status(
                                    task_id,
                                    crate::ai::types::TaskStatus::Cancelled,
                                );
                            } else {
                                task_manager.update_task_status(
                                    task_id,
                                    crate::ai::types::TaskStatus::Failed,
                                );
                            }
                        }
                        Err(_) => {
                            task_manager
                                .update_task_status(task_id, crate::ai::types::TaskStatus::Failed);
                        }
                    }

                    // Notify the UI thread that an update is needed
                    if let Some(tx) = ui_tx {
                        let _ = tx.send(()).await;
                    }

                    result
                });

                // Create a channel to send the response back to the main thread
                let (response_tx, response_rx) = tokio::sync::mpsc::channel::<Option<String>>(1);
                
                // Store the receiver for later use
                self.task_manager.set_response_channel(task_id, response_rx);
                
                // We'll save the result handling in a separate task to avoid blocking
                let ui_tx_clone = self.ui_notifier.clone();
                let result_handler = tokio::spawn(async move {
                    // Await the AI task result
                    let result = ai_task.await;
                    
                    // Process the result to get the AI response content
                    let response_content = match result {
                        Ok(Ok(response)) => {
                            // If we have a successful response, extract the content
                            if let Ok(ai_response) = response {
                                // Quietly return the content without debug messages
                                Some(ai_response.content)
                            } else {
                                None
                            }
                        }
                        _ => {
                            None
                        }
                    };
                    
                    // Send the response content back to the main thread
                    let _ = response_tx.send(response_content).await;
                    
                    // Notify the UI thread that we have a result
                    if let Some(tx) = ui_tx_clone {
                        let _ = tx.send(()).await;
                    }
                });

                // Store the task in our background tasks
                self.background_tasks.push(result_handler);

                // No processing indicator, keep output minimal

                // Set up spinner cleanup when AI task completes
                let ui_tx_clone = self.ui_notifier.clone();
                tokio::spawn(async move {
                    // Give the task some time to run
                    tokio::time::sleep(Duration::from_secs(120)).await;
                    
                    // Abort the spinner task
                    spinner_task.abort();
                    
                    // Notify UI thread that we should refresh
                    if let Some(tx) = ui_tx_clone {
                        let _ = tx.send(()).await;
                    }
                });
            }
        }

        // Update current mode
        self.current_mode = mode;
    }

    pub fn navigate_history_up(&mut self) {
        if self.history.commands.is_empty() {
            return;
        }

        if self.history.position > 0 {
            self.history.position -= 1;
            if let Some(cmd) = self.history.commands.get(self.history.position) {
                self.input = cmd.clone();
            }
        }
    }

    pub fn navigate_history_down(&mut self) {
        if self.history.commands.is_empty() {
            return;
        }

        // Navigate through history with match statement
        match self
            .history
            .position
            .cmp(&(self.history.commands.len() - 1))
        {
            std::cmp::Ordering::Less => {
                // Not at the end of history yet
                self.history.position += 1;
                if let Some(cmd) = self.history.commands.get(self.history.position) {
                    self.input = cmd.clone();
                }
            }
            std::cmp::Ordering::Equal => {
                // At the end of history, clear input
                self.history.position = self.history.commands.len();
                self.input.clear();
            }
            std::cmp::Ordering::Greater => {
                // Already beyond history
            }
        }
    }

    // Text selection and copying functions
    pub fn start_text_selection(&mut self) {
        self.is_selecting_text = true;
        let visible_line = self.scroll_offset as usize;
        self.selection_start = visible_line;
        self.selection_end = visible_line;
    }

    // Mouse-based text selection methods
    pub fn start_mouse_selection(&mut self, x: u16, y: u16) {
        self.mouse_drag_ongoing = true;
        self.mouse_drag_start_x = x;
        self.mouse_drag_start_y = y;

        // Calculate line index based on y position
        let line_idx = self.scroll_offset as usize + y as usize;
        if line_idx < self.output_lines.len() {
            self.is_selecting_text = true;
            self.selection_start = line_idx;
            self.selection_end = line_idx;

            // Check for double click
            let now = Instant::now();
            let double_click_threshold = Duration::from_millis(500); // 500ms for double click

            if now.duration_since(self.last_click_time) < double_click_threshold
                && self.last_click_pos == (x, y)
            {
                // Double click detected - select word
                self.select_word_at(line_idx);
            }

            // Update for future double click detection
            self.last_click_time = now;
            self.last_click_pos = (x, y);
        }
    }

    // Select a word at the given line
    fn select_word_at(&mut self, line_idx: usize) {
        if line_idx >= self.output_lines.len() {
            return;
        }

        // Get the line content - not using it for now, but will in a more advanced implementation
        let _line = &self.output_lines[line_idx];

        // In a more advanced implementation, you would determine
        // the word boundaries based on mouse x position
        // For now, we'll just select the entire line as a simplification
        self.selection_start = line_idx;
        self.selection_end = line_idx;
    }

    pub fn update_mouse_selection(&mut self, _x: u16, y: u16) {
        if !self.mouse_drag_ongoing {
            return;
        }

        // Calculate line index based on y position
        let line_idx = self.scroll_offset as usize + y as usize;
        if line_idx < self.output_lines.len() {
            self.selection_end = line_idx;

            // Auto-scroll if at the edges
            if y == 0 && self.scroll_offset > 0 {
                self.scroll_up(1);
            } else if y >= self.output_area_height.saturating_sub(2) {
                self.scroll_down(1);
            }
        }
    }

    pub fn end_mouse_selection(&mut self) {
        self.mouse_drag_ongoing = false;

        // If start and end are the same, we still maintain selection
        // This allows for clicking on a line to select it
    }

    pub fn cancel_text_selection(&mut self) {
        self.is_selecting_text = false;
    }

    pub fn move_selection_up(&mut self) {
        if self.selection_start > 0 {
            self.selection_start -= 1;
            // Adjust scroll if needed
            if self.selection_start < self.scroll_offset as usize {
                self.scroll_up(1);
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if self.selection_end < self.output_lines.len().saturating_sub(1) {
            self.selection_end += 1;
            // Adjust scroll if needed to keep selection visible
        }
    }

    pub fn copy_selected_text(&mut self) {
        // Ensure start <= end
        let start = self.selection_start.min(self.selection_end);
        let end = self.selection_start.max(self.selection_end);

        // Get the selected text
        let selected_lines = &self.output_lines[start..=end];
        let selected_text = selected_lines.join("\n");

        // Copy to clipboard
        if let Ok(mut ctx) = ClipboardContext::new() {
            if let Err(e) = ctx.set_contents(selected_text) {
                self.add_output(format!("⚠️ Failed to copy to clipboard: {}", e));
            } else {
                self.add_output("✅ Text copied to clipboard".to_string());
            }
        } else {
            self.add_output("⚠️ Failed to access clipboard".to_string());
        }

        // Reset selection
        self.cancel_text_selection();
    }

    pub fn scroll_up(&mut self, amount: u16) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(amount);
        }
    }

    pub fn scroll_down(&mut self, amount: u16) {
        // This will be clamped in the UI rendering if it exceeds the content
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    // Context menu handling
    pub fn show_context_menu(&mut self, x: u16, y: u16) {
        self.show_context_menu = true;
        self.context_menu_x = x;
        self.context_menu_y = y;
    }

    pub fn hide_context_menu(&mut self) {
        self.show_context_menu = false;
    }

    pub fn handle_context_menu_action(&mut self, action: &str) {
        match action {
            "copy" => {
                if self.is_selecting_text {
                    self.copy_selected_text();
                } else {
                    // If nothing is selected, select the line under cursor
                    let line_idx = (self.scroll_offset + self.context_menu_y) as usize;
                    if line_idx < self.output_lines.len() {
                        self.selection_start = line_idx;
                        self.selection_end = line_idx;
                        self.copy_selected_text();
                    }
                }
            }
            "paste" => {
                // Get text from clipboard
                if let Ok(mut ctx) = ClipboardContext::new() {
                    if let Ok(text) = ctx.get_contents() {
                        self.input.push_str(&text);
                    }
                }
            }
            "select_all" => {
                if !self.output_lines.is_empty() {
                    self.is_selecting_text = true;
                    self.selection_start = 0;
                    self.selection_end = self.output_lines.len() - 1;
                }
            }
            _ => {}
        }
        self.hide_context_menu();
    }

    pub fn toggle_selection_mode(&mut self) -> io::Result<()> {
        self.native_selection_mode = !self.native_selection_mode;
        Ok(())
    }
    
    /// Toggle the task popup visibility
    pub fn toggle_tasks_popup(&mut self) {
        self.show_tasks_popup = !self.show_tasks_popup;
    }
    
    /// Get active tasks for display
    pub fn get_active_tasks(&self) -> Vec<crate::utils::tasks::Task> {
        self.task_manager.active_tasks()
    }
    
    /// Get recent completed tasks
    pub fn get_recent_tasks(&self) -> Vec<crate::utils::tasks::Task> {
        self.task_manager.recent_tasks()
    }
    
    /// Check if the cleanup timer has been initialized
    pub fn has_cleanup_timer(&self) -> bool {
        self.last_cleanup_time.is_some()
    }
    
    /// Initialize the cleanup timer
    pub fn init_cleanup_timer(&mut self) {
        self.last_cleanup_time = Some(Instant::now());
    }
    
    /// Check if we should perform a cleanup based on time elapsed
    pub fn should_perform_cleanup(&self) -> bool {
        match self.last_cleanup_time {
            Some(last_time) => {
                let now = Instant::now();
                now.duration_since(last_time).as_secs() > 60 // Cleanup every minute
            }
            None => false,
        }
    }
    
    /// Reset the cleanup timer
    pub fn reset_cleanup_timer(&mut self) {
        self.last_cleanup_time = Some(Instant::now());
    }
    
    /// Cancel a task by ID
    pub fn cancel_task(&mut self, id: crate::utils::tasks::TaskId) -> bool {
        // Get the task first to determine if it's still active
        let task_opt = self.task_manager.get_task(id);
        
        if let Some(task) = task_opt {
            // Only try to cancel if the task is active
            if task.status == crate::ai::types::TaskStatus::Running || 
               task.status == crate::ai::types::TaskStatus::Pending {
                
                // For cancellable background tasks like AI generation
                if task.task_type == crate::utils::tasks::TaskType::AIGeneration {
                    // Set abort flags to stop any running AI operations
                    self.abort_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                    if let Some(global_abort) = &self.global_abort {
                        global_abort.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    
                    // Add a message to the output indicating cancellation was requested
                    self.add_output(format!("\n[Task {}] Cancellation requested.\n", id.short()));
                }
                
                // Mark the task as cancelled in the task manager
                return self.task_manager.cancel_task(id);
            }
        }
        
        false // Task doesn't exist or is already completed
    }

    // Get formatted session cost information for the /cost command
    pub fn get_session_cost_info(&self) -> String {
        // Calculate individual costs
        let (input_cost, output_cost) = if self.stats.total_tokens > 0 {
            let input_ratio = self.stats.prompt_tokens as f64 / self.stats.total_tokens as f64;
            let output_ratio = self.stats.completion_tokens as f64 / self.stats.total_tokens as f64;
            (
                self.stats.cost * input_ratio,
                self.stats.cost * output_ratio,
            )
        } else {
            (0.0, 0.0)
        };

        format!(
            "Session statistics:\n\
            Tokens used:\n\
            - Input: {} tokens\n\
            - Output: {} tokens\n\
            - Total: {} tokens\n\n\
            Cost breakdown:\n\
            - Input cost: ${:.6}\n\
            - Output cost: ${:.6}\n\
            - Total cost: ${:.6}",
            self.stats.prompt_tokens,
            self.stats.completion_tokens,
            self.stats.total_tokens,
            input_cost,
            output_cost,
            self.stats.cost
        )
    }

    // Update cursor blink state and handle spinner updates if needed
    pub fn update_cursor_blink(&mut self) {
        // Blink cursor every 500ms
        const CURSOR_BLINK_RATE_MS: u128 = 500;

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_cursor_toggle).as_millis();

        if elapsed >= CURSOR_BLINK_RATE_MS {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_toggle = now;
        }

        // Check if we received any spinner update from the background thread
        let mut updated = false;
        if let Some(rx) = &self.spinner_rx {
            // Process all pending updates, but only take the latest one
            let mut latest_update = None;
            while let Ok((frame, line_index)) = rx.try_recv() {
                latest_update = Some((frame, line_index));
            }

            // If we got any updates, apply the latest one
            if let Some((frame, line_index)) = latest_update {
                // Update the spinner in the output area
                if line_index < self.output_lines.len() {
                    // Update the line with the new spinner frame
                    self.output_lines[line_index] = frame;

                    // Rebuild the output string to reflect the spinner update
                    // Make sure we use the entire output_lines vector
                    let mut rebuilt_output = String::new();
                    for (i, line) in self.output_lines.iter().enumerate() {
                        rebuilt_output.push_str(line);
                        if i < self.output_lines.len() - 1 || !self.output.ends_with('\n') {
                            rebuilt_output.push('\n');
                        }
                    }
                    self.output = rebuilt_output;
                    
                    updated = true;
                }
            }
        }

        // If we made changes, trigger a redraw
        if updated {
            // The redraw will happen on the next tick naturally
        }
    }

    pub async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        if let Ok(event) = tui.events().next() {
            match event {
                Event::Abort => {
                    // Set both the local and global abort flags immediately
                    self.abort_requested.store(true, std::sync::atomic::Ordering::SeqCst);
                    if let Some(global_abort) = &self.global_abort {
                        // Use SeqCst ordering to ensure all threads see this change immediately
                        global_abort.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                    
                    // Cancel any ongoing operations
                    // Always show abort message in output area (but avoid duplicates)
                    if !self.output.contains("[Operation Aborted]") {
                        self.add_output("\n[Operation Aborted] ❌ Cancellation requested. Processing should stop momentarily.\n".to_string());
                    }
                    
                    // Cancel spinner if it exists - this is critical for releasing resources
                    if self.spinner_rx.is_some() {
                        if let Some(handle) = self.spinner_rx.take() {
                            // Explicitly drop the channel to ensure the spinner task terminates
                            drop(handle);
                        }
                    }
                    
                    // Reset state that may be affected
                    self.is_scrolling = false;
                    
                    // Force immediate UI refresh to show abort message
                    tui.immediate_refresh(|f| {
                        ui::render(f, self);
                    }).ok();
                }
                Event::Tick => {
                    // Update cursor blink state
                    self.update_cursor_blink();
                }
                Event::Key(key_event) => {
                    // Only handle key events if we're not scrolling
                    if !self.is_scrolling {
                        // Hide context menu on any key press
                        if self.show_context_menu {
                            // Handle menu selection
                            if key_event.code == KeyCode::Enter {
                                let menu_options = ["copy", "paste", "select_all"];
                                if let Some(selected) = menu_options.first() {
                                    // In the future, track selected item
                                    self.handle_context_menu_action(selected);
                                }
                                return Ok(());
                            }
                            self.hide_context_menu();
                            return Ok(());
                        }

                        match key_event.code {
                            KeyCode::Enter => {
                                // Check if Shift is held - if so, insert newline instead of submitting
                                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                                    // Insert a newline at cursor position
                                    self.input.insert(self.cursor_position, '\n');
                                    self.cursor_position += 1;
                                    // Reset cursor blink
                                    self.cursor_visible = true;
                                    self.last_cursor_toggle = Instant::now();
                                } else {
                                    // Submit the command
                                    let command = self.input.trim().to_string();
                                    if !command.is_empty() {
                                        self.input.clear();
                                        self.cursor_position = 0;
                                        self.execute_command(command, tui).await;
                                    }
                                }
                            }
                            KeyCode::Char('c')
                                if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                // Handle task cancellation in tasks popup view
                                if self.show_tasks_popup {
                                    // Get the first active task and cancel it
                                    let active_tasks = self.get_active_tasks();
                                    if !active_tasks.is_empty() {
                                        // Cancel the most recent active task
                                        let task_id = active_tasks[0].id;
                                        if self.cancel_task(task_id) {
                                            self.add_output(format!("\nCancelling task {}...\n", task_id.short()));
                                        }
                                    }
                                }
                                // Handle text selection copy
                                else if self.is_selecting_text {
                                    self.copy_selected_text();
                                }
                                // Otherwise abort is handled in Event::Abort handler
                            }
                            // Context menu key
                            KeyCode::Char('k') if key_event.modifiers == KeyModifiers::CONTROL => {
                                self.show_context_menu(10, 10); // Show context menu at center
                            }
                            // Show tasks popup with Ctrl+T
                            KeyCode::Char('t') if key_event.modifiers == KeyModifiers::CONTROL => {
                                self.toggle_tasks_popup();
                            }
                            // Start text selection with Shift+Up/Down
                            KeyCode::Up if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
                                self.start_text_selection();
                                self.move_selection_up();
                            }
                            KeyCode::Down if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
                                self.start_text_selection();
                                self.move_selection_down();
                            }
                            // Normal navigation
                            KeyCode::Up => {
                                self.navigate_history_up();
                            }
                            KeyCode::Down => {
                                self.navigate_history_down();
                            }
                            // Scrolling with page up/down
                            KeyCode::PageUp => {
                                self.scroll_up(10);
                            }
                            KeyCode::PageDown => {
                                self.scroll_down(10);
                            }
                            KeyCode::Char('v') if key_event.modifiers == KeyModifiers::CONTROL => {
                                // Paste from clipboard
                                self.handle_context_menu_action("paste");
                            }
                            KeyCode::Char('a') if key_event.modifiers == KeyModifiers::CONTROL => {
                                // Select all
                                self.handle_context_menu_action("select_all");
                            }
                            // Input editing with cursor support
                            KeyCode::Char(c) => {
                                // Insert character at cursor position
                                self.input.insert(self.cursor_position, c);
                                self.cursor_position += 1;
                                // Reset blink timer and make cursor visible when typing
                                self.cursor_visible = true;
                                self.last_cursor_toggle = Instant::now();
                            }
                            KeyCode::Backspace => {
                                // Delete character before cursor
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                    self.input.remove(self.cursor_position);
                                    // Reset blink timer
                                    self.cursor_visible = true;
                                    self.last_cursor_toggle = Instant::now();
                                }
                            }
                            KeyCode::Delete => {
                                // Delete character at cursor
                                if self.cursor_position < self.input.len() {
                                    self.input.remove(self.cursor_position);
                                    // Reset blink timer
                                    self.cursor_visible = true;
                                    self.last_cursor_toggle = Instant::now();
                                }
                            }
                            // Cursor movement
                            KeyCode::Left => {
                                if self.cursor_position > 0 {
                                    self.cursor_position -= 1;
                                    // Reset blink timer and make cursor visible when moving
                                    self.cursor_visible = true;
                                    self.last_cursor_toggle = Instant::now();
                                }
                            }
                            KeyCode::Right => {
                                if self.cursor_position < self.input.len() {
                                    self.cursor_position += 1;
                                    // Reset blink timer and make cursor visible when moving
                                    self.cursor_visible = true;
                                    self.last_cursor_toggle = Instant::now();
                                }
                            }
                            KeyCode::Home => {
                                self.cursor_position = 0;
                                // Reset blink timer and make cursor visible when moving
                                self.cursor_visible = true;
                                self.last_cursor_toggle = Instant::now();
                            }
                            KeyCode::End => {
                                self.cursor_position = self.input.len();
                                // Reset blink timer and make cursor visible when moving
                                self.cursor_visible = true;
                                self.last_cursor_toggle = Instant::now();
                            }
                            KeyCode::Esc => {
                                // Handle local functions only, the abort is handled at the Event::Abort level
                                if self.show_tasks_popup {
                                    self.show_tasks_popup = false;
                                } else if self.is_selecting_text {
                                    self.cancel_text_selection();
                                } else {
                                    self.input.clear();
                                    self.cursor_position = 0;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Only process mouse events in vim-like selection mode
                    if !self.native_selection_mode {
                        // Only process mouse events in the output area (y < output_area_height)
                        if mouse_event.row < self.output_area_height {
                            match mouse_event.kind {
                                crossterm::event::MouseEventKind::Down(
                                    crossterm::event::MouseButton::Right,
                                ) => {
                                    self.show_context_menu(mouse_event.column, mouse_event.row);
                                }
                                crossterm::event::MouseEventKind::Down(
                                    crossterm::event::MouseButton::Left,
                                ) => {
                                    self.start_mouse_selection(mouse_event.column, mouse_event.row);
                                }
                                crossterm::event::MouseEventKind::Drag(
                                    crossterm::event::MouseButton::Left,
                                ) => {
                                    self.update_mouse_selection(
                                        mouse_event.column,
                                        mouse_event.row,
                                    );
                                }
                                crossterm::event::MouseEventKind::Up(
                                    crossterm::event::MouseButton::Left,
                                ) => {
                                    self.end_mouse_selection();
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Event::Resize(_, _) => {}
                Event::Copy => {
                    if !self.native_selection_mode {
                        self.copy_selected_text();
                    }
                }
                Event::ScrollUp => {
                    self.is_scrolling = true;
                    self.scroll_up(3); // Scroll 3 lines at a time for better UX
                    self.is_scrolling = false;
                }
                Event::ScrollDown => {
                    self.is_scrolling = true;
                    self.scroll_down(3); // Scroll 3 lines at a time for better UX
                    self.is_scrolling = false;
                }
            }
        }
        Ok(())
    }
}