//! UI components module
//!
//! Provides reusable UI components for the terminal interface

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// Renders a popup message box
#[allow(dead_code)]
pub fn render_popup(
    f: &mut Frame,
    title: &str,
    content: &str,
    width: u16,
    height: u16,
    primary_color: Color,
    background_color: Color,
) {
    // Calculate center position
    let size = f.size();
    let popup_x = (size.width.saturating_sub(width)) / 2;
    let popup_y = (size.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, width, height);

    // Create block with border
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(primary_color))
        .style(Style::default().bg(background_color));

    // Create text content
    let content_text = Text::from(content);

    // Create widget
    let popup_widget = Paragraph::new(content_text)
        .block(popup_block)
        .wrap(Wrap { trim: true });

    // Render popup over everything else
    f.render_widget(popup_widget, popup_area);
}

/// Renders a list selection popup
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn render_list_popup<T: AsRef<str>>(
    f: &mut Frame,
    title: &str,
    items: &[T],
    state: &mut ListState,
    width: u16,
    height: u16,
    primary_color: Color,
    background_color: Color,
) {
    // Calculate center position
    let size = f.size();
    let popup_x = (size.width.saturating_sub(width)) / 2;
    let popup_y = (size.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, width, height);

    // Create block with border
    let popup_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(primary_color))
        .style(Style::default().bg(background_color));

    // Create list items
    let list_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(i.as_ref().to_string()))
        .collect();

    // Create list widget
    let list = List::new(list_items)
        .block(popup_block)
        .highlight_style(Style::default().bg(primary_color).fg(background_color));

    // Render list with state
    f.render_stateful_widget(list, popup_area, state);
}

/// Renders a help overlay
#[allow(dead_code)]
pub fn render_help_overlay(f: &mut Frame, background_color: Color, text_color: Color) {
    // Cover the entire screen
    let area = f.size();

    // Create semi-transparent background
    let overlay_block =
        Block::default().style(Style::default().bg(background_color).fg(text_color));

    // Help content
    let help_text = "
    KEYBOARD SHORTCUTS
    ------------------
    Up/Down: Navigate history
    Shift+Up/Down: Select text
    Ctrl+C: Copy selection or exit
    PageUp/Down: Scroll output
    Ctrl+K: Show context menu
    Esc: Cancel selection
    
    COMMAND PREFIXES
    ---------------
    No prefix: AI mode
    !: Bash command
    /: Application command
    
    Press ESC to close help
    ";

    let help_widget = Paragraph::new(help_text)
        .block(overlay_block)
        .wrap(Wrap { trim: true });

    f.render_widget(help_widget, area);
}

/// Renders a loading indicator
#[allow(dead_code)]
pub fn render_loading(f: &mut Frame, message: &str, accent_color: Color, background_color: Color) {
    // Small popup for loading
    let width = message.len() as u16 + 10;
    let height = 3;

    let size = f.size();
    let popup_x = (size.width.saturating_sub(width)) / 2;
    let popup_y = (size.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, width, height);

    // Create block with border
    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent_color))
        .style(Style::default().bg(background_color));

    // Create loading text
    let loading_text = Text::from(format!("⏳ {}", message));

    // Create widget
    let popup_widget = Paragraph::new(loading_text)
        .block(popup_block)
        .wrap(Wrap { trim: true });

    // Render popup over everything else
    f.render_widget(popup_widget, popup_area);
}
