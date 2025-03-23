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
    Ctrl+T: Show active tasks
    Esc: Cancel selection / Close popup
    
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

/// Renders a tasks popup displaying active and recent tasks
pub fn render_tasks_popup(
    f: &mut Frame,
    app: &crate::app::App,
    primary_color: Color,
    accent_color: Color,
    background_color: Color,
) {
    use crate::ai::types::TaskStatus;
    use ratatui::layout::{Constraint, Direction, Layout};

    // Get tasks
    let active_tasks = app.get_active_tasks();
    let recent_tasks = app.get_recent_tasks();

    // Determine popup size - adjust based on content
    let width = 70.min(f.size().width.saturating_sub(4));
    let height = 20.min(f.size().height.saturating_sub(4));

    // Calculate center position
    let size = f.size();
    let popup_x = (size.width.saturating_sub(width)) / 2;
    let popup_y = (size.height.saturating_sub(height)) / 2;

    let popup_area = Rect::new(popup_x, popup_y, width, height);

    // Create block with border
    let popup_block = Block::default()
        .title("Background Tasks")
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(primary_color))
        .style(Style::default().bg(background_color));

    // Split into sections for active and recent tasks
    let inner_area = popup_block.inner(popup_area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Length(if active_tasks.is_empty() {
                1
            } else {
                active_tasks.len().min(5) as u16 + 2
            }), // Active tasks
            Constraint::Length(1), // Separator
            Constraint::Min(5),    // Recent tasks
            Constraint::Length(1), // Footer
        ])
        .split(inner_area);

    // Create headers
    let header = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled("  Status  ", Style::default().fg(accent_color)),
        ratatui::text::Span::raw(" │ "),
        ratatui::text::Span::styled("  Type  ", Style::default().fg(accent_color)),
        ratatui::text::Span::raw(" │ "),
        ratatui::text::Span::styled("  Progress  ", Style::default().fg(accent_color)),
        ratatui::text::Span::raw(" │ "),
        ratatui::text::Span::styled("  Task  ", Style::default().fg(accent_color)),
    ]);

    // Create active tasks section
    let active_header = ratatui::text::Line::from(vec![ratatui::text::Span::styled(
        "ACTIVE TASKS",
        Style::default().fg(accent_color),
    )]);

    let mut active_task_lines = Vec::new();
    if active_tasks.is_empty() {
        active_task_lines.push(ratatui::text::Line::from("  No active tasks"));
    } else {
        for task in &active_tasks {
            let status_style = match task.status {
                TaskStatus::Running => Style::default().fg(Color::Green),
                TaskStatus::Pending => Style::default().fg(Color::Yellow),
                TaskStatus::Completed => Style::default().fg(Color::Blue),
                TaskStatus::Failed => Style::default().fg(Color::Red),
                TaskStatus::Cancelled => Style::default().fg(Color::DarkGray),
            };

            // Format status
            let status_text = format!("  {}  ", task.status);

            // Format type
            let type_text = format!("  {}  ", task.task_type);

            // Format progress
            let progress_text = if let Some(progress) = &task.progress {
                if let Some(percent) = progress.completion_percent {
                    if task.status == TaskStatus::Running {
                        format!(
                            " {:.1}% ({}/s) ",
                            percent, progress.tokens_per_second as u32
                        )
                    } else {
                        format!(" {:.1}% ", percent)
                    }
                } else {
                    format!(" {} tkns ", progress.tokens_generated)
                }
            } else {
                "   -   ".to_string()
            };

            // Format task name with id
            let task_text = format!("  {} ({})", task.name, task.id.short());

            active_task_lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(status_text, status_style),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(type_text),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(progress_text),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(task_text),
            ]));
        }
    }

    // Create recent tasks section
    let recent_header = ratatui::text::Line::from(vec![ratatui::text::Span::styled(
        "RECENTLY COMPLETED",
        Style::default().fg(accent_color),
    )]);

    let mut recent_task_lines = Vec::new();
    if recent_tasks.is_empty() {
        recent_task_lines.push(ratatui::text::Line::from("  No recent tasks"));
    } else {
        for task in &recent_tasks {
            let status_style = match task.status {
                TaskStatus::Completed => Style::default().fg(Color::Blue),
                TaskStatus::Failed => Style::default().fg(Color::Red),
                TaskStatus::Cancelled => Style::default().fg(Color::DarkGray),
                _ => Style::default(),
            };

            // Format status
            let status_text = format!("  {}  ", task.status);

            // Format type
            let type_text = format!("  {}  ", task.task_type);

            // Format duration
            let duration_text = format!(" {} ", task.format_duration());

            // Format task name with id
            let task_text = format!("  {} ({})", task.name, task.id.short());

            recent_task_lines.push(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(status_text, status_style),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(type_text),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(duration_text),
                ratatui::text::Span::raw(" │ "),
                ratatui::text::Span::raw(task_text),
            ]));
        }
    }

    // Create footer
    let footer = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled(
            " ESC ",
            Style::default().bg(accent_color).fg(background_color),
        ),
        ratatui::text::Span::raw(" Close  "),
        ratatui::text::Span::styled(
            " Ctrl+C ",
            Style::default().bg(accent_color).fg(background_color),
        ),
        ratatui::text::Span::raw(" Cancel task"),
    ]);

    // Render popup
    f.render_widget(popup_block, popup_area);

    // Render header
    f.render_widget(Paragraph::new(header), chunks[0]);

    // Render active tasks section
    let active_header_area = Rect::new(chunks[1].x, chunks[1].y, chunks[1].width, 1);
    f.render_widget(Paragraph::new(active_header), active_header_area);
    f.render_widget(
        Paragraph::new(active_task_lines),
        Rect::new(
            chunks[1].x,
            chunks[1].y + 1,
            chunks[1].width,
            chunks[1].height - 1,
        ),
    );

    // Render separator
    f.render_widget(Paragraph::new("─".repeat(width as usize - 2)), chunks[2]);

    // Render recent tasks section
    let recent_header_area = Rect::new(chunks[3].x, chunks[3].y, chunks[3].width, 1);
    f.render_widget(Paragraph::new(recent_header), recent_header_area);
    f.render_widget(
        Paragraph::new(recent_task_lines),
        Rect::new(
            chunks[3].x,
            chunks[3].y + 1,
            chunks[3].width,
            chunks[3].height - 1,
        ),
    );

    // Render footer
    f.render_widget(Paragraph::new(footer), chunks[4]);
}
