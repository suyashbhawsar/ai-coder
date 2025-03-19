//! UI module for rendering the terminal interface
//!
//! This module handles all UI rendering aspects, including:
//! - Main layout
//! - Input/output areas
//! - Status bar
//! - Context menus
//! - Help overlay

use std::path::Path;
use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::config::{get_config, ThemeConfig};

mod components;
mod theme;
pub use theme::Theme;

/// Convert hex color to ratatui Color
fn parse_hex_color(hex: &str) -> Color {
    if hex == "default" {
        return Color::Reset;
    }

    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Reset;
    }

    if let (Ok(r), Ok(g), Ok(b)) = (
        u8::from_str_radix(&hex[0..2], 16),
        u8::from_str_radix(&hex[2..4], 16),
        u8::from_str_radix(&hex[4..6], 16),
    ) {
        Color::Rgb(r, g, b)
    } else {
        Color::Reset
    }
}

/// Get colors from theme config
pub fn get_theme_colors(theme: &ThemeConfig) -> (Color, Color, Color, Color, Color) {
    let primary = parse_hex_color(&theme.primary);
    let secondary = parse_hex_color(&theme.secondary);
    let accent = parse_hex_color(&theme.accent);
    let background = parse_hex_color(&theme.background);
    let foreground = parse_hex_color(&theme.foreground);
    
    (primary, secondary, accent, background, foreground)
}

/// Main render function
pub fn render(f: &mut Frame, app: &mut App) {
    // Get terminal size
    let size = f.size();
    
    // Get theme from config
    let config = get_config();
    let (primary, _secondary, accent, background, foreground) = get_theme_colors(&config.theme);

    // Calculate input area height accounting for both explicit newlines and wrapping
    // First count explicit newlines
    let explicit_lines = app.input.lines().count();
    
    // Calculate approximate wrapped lines based on terminal width
    // Subtract 4 from width to account for borders and prompt prefix
    let content_width = size.width.saturating_sub(4) as usize;
    
    // Calculate wrapped lines more accurately by considering line breaks
    let mut wrapped_lines = 0;
    if content_width > 0 {
        for line in app.input.lines() {
            // For each line, calculate how many wrapped lines it would need
            // +1 ensures we round up, so even a partial line gets counted
            let line_wraps = (line.chars().count() + 1) / content_width + 1;
            wrapped_lines += line_wraps;
        }
    } else {
        wrapped_lines = 1;
    };
    
    // If input is empty, ensure at least one line
    if app.input.is_empty() {
        wrapped_lines = 1;
    }
    
    // Use the larger of explicit newlines or wrapped lines calculation
    let estimated_lines = explicit_lines.max(wrapped_lines);
    
    // Ensure input box is at least 3 lines tall and doesn't exceed 10 lines
    // (adjust the max as needed based on your preferences)
    let input_height = (estimated_lines as u16).max(1).min(10) + 2; // Add 2 for border
    
    // Create the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                  // Output area (takes all remaining space)
            Constraint::Length(input_height),    // Input area (flexible height)
            Constraint::Length(1),               // Status bar (fixed height)
        ])
        .split(size);

    // Render each component
    render_output_area(f, app, chunks[0], background, foreground);
    render_input_area(f, app, chunks[1], background, foreground);
    render_status_bar(f, app, chunks[2], primary, accent, background);

    // Store output area height for mouse handling
    app.output_area_height = chunks[0].height;

    // Render context menu if active
    if app.show_context_menu {
        render_context_menu(f, app, accent, background, foreground);
    }
}

/// Render the context menu
fn render_context_menu(f: &mut Frame, app: &App, accent: Color, bg_color: Color, fg_color: Color) {
    let menu_width = 20;
    let menu_height = 3;
    let menu_x = app.context_menu_x.min(f.size().width.saturating_sub(menu_width));
    let menu_y = app.context_menu_y.min(f.size().height.saturating_sub(menu_height));

    let menu_area = Rect::new(menu_x, menu_y, menu_width, menu_height);

    let menu_block = Block::default()
        .title("Actions")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(bg_color));

    let menu_text = vec![
        Line::from("Copy"),
        Line::from("Select All"),
        Line::from("Clear"),
    ];

    let menu_widget = Paragraph::new(menu_text)
        .block(menu_block)
        .style(Style::default().fg(fg_color));

    f.render_widget(menu_widget, menu_area);
}

/// Render the output area
fn render_output_area(f: &mut Frame, app: &App, area: Rect, bg_color: Color, fg_color: Color) {
    // No border for output area as requested
    let output_block = Block::default()
        .style(Style::default().bg(bg_color).fg(fg_color));

    // Create styled text with selection highlighting if applicable
    let mut styled_lines = Vec::new();

    // Only show custom selection highlighting in vim-like mode
    if app.is_selecting_text && !app.native_selection_mode {
        let start = app.selection_start.min(app.selection_end);
        let end = app.selection_start.max(app.selection_end);

        for (idx, line) in app.output_lines.iter().enumerate() {
            if idx >= start && idx <= end {
                // Highlighted selection
                styled_lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::default().bg(Color::White).fg(Color::Black)
                )));
            } else {
                // Normal text
                styled_lines.push(Line::from(Span::raw(line.clone())));
            }
        }
    } else {
        // Regular rendering without selection
        let text = Text::from(app.output.clone());
        let lines = text.lines.to_vec();
        styled_lines = lines;
    }

    let text = Text::from(styled_lines);

    let output_widget = Paragraph::new(text)
        .block(output_block)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset, 0));

    f.render_widget(output_widget, area);
}

/// Render the input area
fn render_input_area(f: &mut Frame, app: &App, area: Rect, bg_color: Color, fg_color: Color) {
    let input_block = Block::default()
        .title("Input")
        .borders(Borders::ALL)
        .style(Style::default().bg(bg_color).fg(fg_color));

    // Calculate the inner area to help with text wrapping and scroll management
    let _inner_area = input_block.inner(area);
    
    // Determine cursor style - vertical bar when visible
    let cursor_style = if app.cursor_visible {
        Style::default().fg(fg_color).add_modifier(ratatui::style::Modifier::REVERSED)
    } else {
        Style::default().fg(fg_color) // Same as normal text when invisible
    };
    
    // Create the text object with proper cursor support
    let mut text = Text::default();
    
    // For multiline input, split by newlines first
    let input_parts: Vec<&str> = app.input.split('\n').collect();
    let mut current_pos = 0;
    
    for (i, part) in input_parts.iter().enumerate() {
        let part_len = part.len() + if i < input_parts.len() - 1 { 1 } else { 0 }; // +1 for the newline
        
        // Check if cursor is in this line segment
        if current_pos <= app.cursor_position && app.cursor_position < current_pos + part_len {
            // Cursor is in this segment
            let line_cursor_pos = app.cursor_position - current_pos;
            
            // Create spans for this line
            let mut spans = Vec::new();
            
            // Add the prompt only to the first line
            if i == 0 {
                spans.push(Span::raw("> "));
            } else {
                spans.push(Span::raw("  ")); // Indent continuation lines
            }
            
            // Add text before cursor
            if line_cursor_pos > 0 {
                spans.push(Span::raw(&part[..line_cursor_pos]));
            }
            
            // Add cursor or character at cursor
            if line_cursor_pos < part.len() {
                let cursor_char = part[line_cursor_pos..].chars().next().unwrap_or(' ');
                if app.cursor_visible {
                    spans.push(Span::styled(cursor_char.to_string(), cursor_style));
                } else {
                    spans.push(Span::raw(cursor_char.to_string()));
                }
                
                // Add text after cursor
                if line_cursor_pos + 1 < part.len() {
                    spans.push(Span::raw(&part[line_cursor_pos + 1..]));
                }
            } else {
                // Cursor at end of line
                if app.cursor_visible {
                    spans.push(Span::styled("│", cursor_style));
                }
            }
            
            text.lines.push(Line::from(spans));
        } else {
            // Cursor not in this segment, render normally
            let mut spans = Vec::new();
            if i == 0 {
                spans.push(Span::raw("> "));
            } else {
                spans.push(Span::raw("  ")); // Indent continuation lines
            }
            spans.push(Span::raw(*part));
            text.lines.push(Line::from(spans));
        }
        
        current_pos += part_len;
    }
    
    // If cursor is at the very end and there's no newline at the end
    if app.cursor_position == app.input.len() && (app.input.is_empty() || !app.input.ends_with('\n')) {
        // If the text is empty or we haven't added any lines yet
        if text.lines.is_empty() {
            let mut spans = Vec::new();
            spans.push(Span::raw("> "));
            if app.cursor_visible {
                spans.push(Span::styled("│", cursor_style));
            }
            text.lines.push(Line::from(spans));
        } else {
            // Get the last line
            let last_idx = text.lines.len() - 1;
            let last_line = &text.lines[last_idx];
            
            // Add cursor at the end of the last line if it's visible
            if app.cursor_visible {
                let mut new_spans = last_line.spans.clone();
                new_spans.push(Span::styled("│", cursor_style));
                text.lines[last_idx] = Line::from(new_spans);
            }
        }
    }
    
    let input_widget = Paragraph::new(text)
        .block(input_block)
        .style(Style::default().fg(fg_color))
        .wrap(Wrap { trim: false }); // Enable wrapping for multiline input

    // Render the input widget
    f.render_widget(input_widget, area);
}

/// Render the status bar
fn render_status_bar(
    f: &mut Frame,
    app: &App,
    area: Rect,
    primary_color: Color,
    accent_color: Color,
    bg_color: Color,
) {
    let elapsed = Local::now() - app.stats.start_time;
    let hours = elapsed.num_hours();
    let minutes = elapsed.num_minutes() % 60;
    let seconds = elapsed.num_seconds() % 60;
    let elapsed_str = format!("{}h {}m {}s", hours, minutes, seconds);

    // Current directory name
    let current_dir = Path::new(&app.current_dir);
    let dir_name = current_dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from(""));

    // Style for mode indicator
    let mode_style = Style::default().bg(accent_color).fg(bg_color);

    // Create status bar spans
    let mut spans = vec![
        Span::styled(format!(" {} ", app.current_mode), mode_style),
        Span::raw(" "),
        Span::raw(format!("📁 {} ", dir_name)),
        Span::raw(" "),
        Span::raw(format!("⏱️ {} ", elapsed_str)),
        Span::raw(" "),
        Span::raw(format!("💰 ${:.4} ", app.stats.cost)),
        Span::raw(" "),
        Span::raw(format!("🧮 {} cmds ", app.stats.command_count)),
    ];

    // Add text selection indicator if applicable
    if app.is_selecting_text {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(" SELECTING ", Style::default().bg(Color::Yellow).fg(Color::Black)));
    }

    let status_text = Line::from(spans);

    let status_widget = Paragraph::new(status_text)
        .style(Style::default().bg(primary_color).fg(bg_color));

    f.render_widget(status_widget, area);
}