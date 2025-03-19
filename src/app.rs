use crate::handlers::{bash, command};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::time::{Duration, Instant};
use anyhow::Result;
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::env;
use std::path::PathBuf;
use std::io;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::event::Event;
use crate::tui::Tui;
use crate::handlers::CommandMode;
use crate::utils::Colors;

mod ai_handler;
use ai_handler::AIHandler;
use crate::ai::AIError;

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
    pub cursor_position: usize, // Track cursor position in input
    pub cursor_visible: bool, // Toggle for cursor blinking
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
    pub output_area_height: u16, // To track output area dimensions
    pub last_click_time: Instant, // For double click detection
    pub last_click_pos: (u16, u16), // For double click detection
    pub native_selection_mode: bool,
    pub is_scrolling: bool, // Track when scrolling is in progress
    pub ai_handler: AIHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            input: String::new(),
            cursor_position: 0, // Initialize cursor at beginning of input
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
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_output(&mut self, text: String) {
        // Ensure the text ends with a newline
        let text = if text.ends_with('\n') { text } else { text + "\n" };
        self.output.push_str(&text);

        // Update output_lines for text selection and copying
        let new_lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        self.output_lines.extend(new_lines);
        if text.ends_with('\n') && !text.is_empty() {
            self.output_lines.push(String::new());
        }
    }

    pub fn format_timestamp(&self) -> String {
        format!("[{}]", Local::now().format("%H:%M:%S"))
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

    pub async fn execute_command(&mut self, command: String) {
        // Add command to history
        self.history.add(command.clone());

        // Detect mode and get processed command
        let (mode, cmd) = self.detect_mode(&command);

        // Add timestamp and command to output
        self.add_output(format!("{} {}", self.format_timestamp(), command));

        match mode {
            CommandMode::Bash => {
                let result = bash::handle_bash_command(&cmd)
                    .unwrap_or_else(|e| format!("Error: {}", e));
                self.add_output(result);
                self.stats.bash_count += 1;
            }
            CommandMode::Command => {
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

                let result = command::CommandHandler::handle_command(&cmd)
                    .unwrap_or_else(|e| format!("Error: {}", e));
                self.add_output(result);
                self.stats.command_count += 1;
            }
            CommandMode::AI => {
                // Add thinking indicator
                self.add_output("⏳ Processing...".to_string());

                // Call the AI handler
                match self.ai_handler.generate(&cmd).await {
                    Ok(response) => {
                        // Update stats
                        self.stats.ai_count += 1;
                        self.stats.total_tokens = self.stats.total_tokens
                            .checked_add(response.usage.total_tokens)
                            .unwrap_or(self.stats.total_tokens);
                            
                        // Update prompt and completion tokens in SessionStats
                        self.stats.prompt_tokens = self.stats.prompt_tokens
                            .checked_add(response.usage.prompt_tokens)
                            .unwrap_or(self.stats.prompt_tokens);
                            
                        self.stats.completion_tokens = self.stats.completion_tokens
                            .checked_add(response.usage.completion_tokens)
                            .unwrap_or(self.stats.completion_tokens);
                        
                        // Get correct model costs based on the model
                        let model_costs = self.ai_handler.get_model_costs(&response.model).await;
                        
                        // Calculate input cost
                        let input_cost = (response.usage.prompt_tokens as f64 / 1000.0) * model_costs.prompt_cost_per_1k;
                        
                        // Calculate output cost
                        let output_cost = (response.usage.completion_tokens as f64 / 1000.0) * model_costs.completion_cost_per_1k;
                        
                        // Update total cost
                        self.stats.cost += input_cost + output_cost;
                        
                        // Note: Token usage logging has been removed as requested
                        
                        // Remove the thinking indicator and add response
                        self.add_output(response.content);
                    }
                    Err(e) => {
                        // More user-friendly error message
                        match e {
                            AIError::NetworkError(msg) if msg.contains("Ollama not available") => {
                                self.add_output("❌ Error: Ollama service not running. Please start Ollama with 'ollama serve' command.".to_string());
                                self.add_output("📝 If you don't have Ollama installed, visit https://ollama.com to download and install it.".to_string());
                            },
                            AIError::APIError(msg) if msg.contains("operation timed out") => {
                                self.add_output("❌ Error: Connection to Ollama timed out.".to_string());
                                self.add_output("📝 This may happen if:".to_string());
                                self.add_output("  • The model is still loading (especially first use)".to_string());
                                self.add_output("  • Your system is low on RAM or GPU resources".to_string());
                                self.add_output("  • Ollama is processing another request".to_string());
                                self.add_output("Try again in a moment or use a smaller model.".to_string());
                            },
                            _ => self.add_output(format!("❌ Error: {}", e))
                        }
                    }
                }
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
        match self.history.position.cmp(&(self.history.commands.len() - 1)) {
            std::cmp::Ordering::Less => {
                // Not at the end of history yet
                self.history.position += 1;
                if let Some(cmd) = self.history.commands.get(self.history.position) {
                    self.input = cmd.clone();
                }
            },
            std::cmp::Ordering::Equal => {
                // At the end of history, clear input
                self.history.position = self.history.commands.len();
                self.input.clear();
            },
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

            if now.duration_since(self.last_click_time) < double_click_threshold &&
               self.last_click_pos == (x, y) {
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
            },
            "paste" => {
                // Get text from clipboard
                if let Ok(mut ctx) = ClipboardContext::new() {
                    if let Ok(text) = ctx.get_contents() {
                        self.input.push_str(&text);
                    }
                }
            },
            "select_all" => {
                if !self.output_lines.is_empty() {
                    self.is_selecting_text = true;
                    self.selection_start = 0;
                    self.selection_end = self.output_lines.len() - 1;
                }
            },
            _ => {}
        }
        self.hide_context_menu();
    }

    pub fn toggle_selection_mode(&mut self) -> io::Result<()> {
        self.native_selection_mode = !self.native_selection_mode;
        Ok(())
    }
    
    // Get formatted session cost information for the /cost command
    pub fn get_session_cost_info(&self) -> String {
        // Calculate individual costs
        let (input_cost, output_cost) = if self.stats.total_tokens > 0 {
            let input_ratio = self.stats.prompt_tokens as f64 / self.stats.total_tokens as f64;
            let output_ratio = self.stats.completion_tokens as f64 / self.stats.total_tokens as f64;
            (
                self.stats.cost * input_ratio,
                self.stats.cost * output_ratio
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

    // Update cursor blink state
    pub fn update_cursor_blink(&mut self) {
        // Blink cursor every 500ms
        const CURSOR_BLINK_RATE_MS: u128 = 500;
        
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_cursor_toggle).as_millis();
        
        if elapsed >= CURSOR_BLINK_RATE_MS {
            self.cursor_visible = !self.cursor_visible;
            self.last_cursor_toggle = now;
        }
    }

    pub async fn handle_events(&mut self, tui: &mut Tui) -> Result<()> {
        if let Ok(event) = tui.events().next() {
            match event {
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
                                if let Some(selected) = menu_options.first() { // In the future, track selected item
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
                                        self.execute_command(command).await;
                                    }
                                }
                            }
                            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                                if self.is_selecting_text {
                                    self.copy_selected_text();
                                } else {
                                    self.running = false;
                                }
                            }
                            // Context menu key
                            KeyCode::Char('k') if key_event.modifiers == KeyModifiers::CONTROL => {
                                self.show_context_menu(10, 10); // Show context menu at center
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
                                if self.is_selecting_text {
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
                                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                                    self.show_context_menu(mouse_event.column, mouse_event.row);
                                },
                                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                                    self.start_mouse_selection(mouse_event.column, mouse_event.row);
                                },
                                crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                                    self.update_mouse_selection(mouse_event.column, mouse_event.row);
                                },
                                crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                                    self.end_mouse_selection();
                                },
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
