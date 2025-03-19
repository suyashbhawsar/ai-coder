//! AI Coder Interface - Main entry point
//!
//! This is the main entry point for the AI Coder Interface application.
//! It initializes the application and runs the main event loop.

use anyhow::Result;
use ai_coder_interface_rs::{App, Tui, init, cleanup};
use ai_coder_interface_rs::utils::{log_info, log_error};

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
    
    // Create application instance
    let mut app = App::new();
    
    // Initialize terminal with 250ms tick rate
    let mut tui = Tui::new(250)?;
    
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
        
        // Handle events using the app's event handler
        if let Err(e) = app.handle_events(&mut tui).await {
            log_error(&format!("Error handling events: {}", e)).ok();
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