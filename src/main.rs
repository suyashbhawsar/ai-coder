//! AI Coder Interface - Main entry point
//!
//! This is the main entry point for the AI Coder Interface application.
//! It initializes the application and runs the main event loop.

use ai_coder_interface_rs::utils::{log_error, log_info};
use ai_coder_interface_rs::{App, Tui, cleanup, init};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize application
    if let Err(e) = init() {
        eprintln!("Failed to initialize application: {}", e);
        return Err(e);
    }

    // Log application start
    log_info("Application started").ok();

    // Create a global abort flag
    let global_abort = Arc::new(AtomicBool::new(false));
    
    // Set up a signal handler to react to Ctrl+C
    let abort_flag_clone = global_abort.clone();
    ctrlc::set_handler(move || {
        abort_flag_clone.store(true, Ordering::SeqCst);
        println!("Abort requested via Ctrl+C");
    }).expect("Error setting Ctrl+C handler");
    
    // Create application instance with abort flag
    let mut app = App::new();
    app.set_global_abort(global_abort);

    // Initialize terminal with 250ms tick rate
    let mut tui = Tui::new(250)?;

    // Create a channel for UI updates
    let (ui_tx, mut ui_rx) = tokio::sync::mpsc::channel::<()>(32);
    app.ui_notifier = Some(ui_tx);
    
    // Create a task update channel
    let mut task_rx = app.task_manager.get_update_receiver();
    
    // Display welcome message
    app.add_output(format!(
        "🚀 AI Coder Interface\nCurrent directory: {}\n",
        std::env::current_dir()?.display()
    ));
    
    // Start the main loop
    while app.running {
        // Render UI
        tui.draw(|f| {
            ai_coder_interface_rs::ui::render(f, &mut app);
        })?;

        // Set up concurrent handling of events, UI updates and background tasks
        tokio::select! {
            // Handle user input events with a timeout to keep UI responsive
            event_result = tokio::time::timeout(tokio::time::Duration::from_millis(50), app.handle_events(&mut tui)) => {
                match event_result {
                    Ok(result) => {
                        if let Err(e) = result {
                            log_error(&format!("Error handling events: {}", e)).ok();
                        }
                    },
                    Err(_) => {
                        // Timeout is expected and helps keep the UI responsive
                    }
                }
            },
            
            // Process any UI update messages
            _ = ui_rx.recv() => {
                // UI update requested, nothing specific to do as we'll redraw at the start of the loop
            },
            
            // Process task updates
            Some(task_id) = task_rx.recv() => {
                // Task update received, check for status changes
                if let Some(task) = app.task_manager.get_task(task_id) {
                    // For completed tasks, add a notification to the output
                    if task.status == ai_coder_interface_rs::ai::types::TaskStatus::Completed ||
                       task.status == ai_coder_interface_rs::ai::types::TaskStatus::Failed ||
                       task.status == ai_coder_interface_rs::ai::types::TaskStatus::Cancelled {
                        // Only notify for AI generation tasks
                        if task.task_type == ai_coder_interface_rs::utils::tasks::TaskType::AIGeneration {
                            let status_str = match task.status {
                                ai_coder_interface_rs::ai::types::TaskStatus::Completed => "✅ Completed",
                                ai_coder_interface_rs::ai::types::TaskStatus::Failed => "❌ Failed",
                                ai_coder_interface_rs::ai::types::TaskStatus::Cancelled => "⚠️ Cancelled",
                                _ => ""
                            };
                            
                            // Silent completion - no message
                            
                            // If this is a completed AI task, process the response
                            if task.status == ai_coder_interface_rs::ai::types::TaskStatus::Completed && 
                               task.task_type == ai_coder_interface_rs::utils::tasks::TaskType::AIGeneration {
                                // Get the response channel for this task
                                if let Some(mut rx) = app.task_manager.take_response_channel(task_id) {
                                    // Try to receive the response (non-blocking)
                                    if let Ok(Some(response_content)) = rx.try_recv() {
                                        // Find and remove all spinner characters from the output
                                        for spinner_char in ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"] {
                                            // Replace spinner in output lines
                                            for i in 0..app.output_lines.len() {
                                                if app.output_lines[i].trim() == spinner_char {
                                                    app.output_lines[i] = "".to_string();
                                                }
                                            }
                                            
                                            // Also replace in the main output string
                                            app.output = app.output.replace(spinner_char, "");
                                        }
                                        
                                        // Replace any double newlines that might have been created
                                        app.output = app.output.replace("\n\n\n", "\n\n");
                                        
                                        // Add the response with a single newline
                                        app.add_output(format!("{}", response_content));
                                    }
                                }
                            }
                        }
                    }
                }
                // Redraw will happen at the start of the next loop
            },
            
            // Add an explicit small delay to prevent CPU hogging
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) => {
                // This represents roughly 60fps and gives other tasks time to run
                app.update_cursor_blink(); // Update cursor blinking state
                
                // Cleanup any completed background tasks
                app.background_tasks.retain(|task| !task.is_finished());
                
                // Clean up old tasks from task manager periodically
                // Initialize a timer if it doesn't exist yet
                if !app.has_cleanup_timer() {
                    app.init_cleanup_timer();
                }
                
                // Check if we need to perform cleanup (every 60 seconds)
                if app.should_perform_cleanup() {
                    // Clean up tasks older than 30 minutes
                    app.task_manager.cleanup_old_tasks();
                    // Reset the timer
                    app.reset_cleanup_timer();
                }
            }
        }
    }

    // Log application exit
    log_info("Application exiting normally").ok();

    // Exit the terminal interface
    tui.exit()?;

    // Clean up resources
    cleanup()?;

    Ok(())
}