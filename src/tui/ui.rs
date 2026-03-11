use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::types::Tag;

use super::{App, AppMode};

pub fn draw(frame: &mut Frame, app: &App) {
    let [top, main, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(frame.area());

    draw_top_bar(frame, app, top);

    match app.mode {
        AppMode::Detail => draw_detail(frame, app, main),
        _ => draw_list(frame, app, main),
    }

    draw_bottom_bar(frame, app, bottom);
}

fn draw_top_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![Span::styled(
        " todohawk ",
        Style::new().bold().fg(Color::White),
    )];

    spans.push(Span::raw(" "));

    for tag in Tag::all() {
        let active = app.active_tag_filters.is_empty() || app.active_tag_filters.contains(tag);
        let style = if active {
            Style::new().fg(tag.color()).bold()
        } else {
            Style::new().fg(Color::DarkGray)
        };
        spans.push(Span::styled(
            format!("[{}:{}]", tag.shortcut_key(), tag),
            style,
        ));
        spans.push(Span::raw(" "));
    }

    if app.mode == AppMode::Search {
        spans.push(Span::styled("/", Style::new().fg(Color::Yellow)));
        spans.push(Span::styled(
            &app.search_query,
            Style::new().fg(Color::White),
        ));
        spans.push(Span::styled(
            "_",
            Style::new()
                .fg(Color::Yellow)
                .add_modifier(Modifier::SLOW_BLINK),
        ));
    } else if !app.search_query.is_empty() {
        spans.push(Span::styled(
            format!(" search: \"{}\"", app.search_query),
            Style::new().fg(Color::DarkGray),
        ));
    }

    let line = Line::from(spans);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.filtered_items.is_empty() {
        let msg = Paragraph::new("  No matching items.")
            .style(Style::new().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(msg, area);
        return;
    }

    let header = Row::new(["Tag", "File", "Line", "Author", "Message"])
        .style(Style::new().bold().fg(Color::White))
        .bottom_margin(0);

    let rows: Vec<Row> = app
        .filtered_items
        .iter()
        .map(|&idx| {
            let item = &app.items[idx];
            let tag_cell =
                Cell::from(item.tag.to_string()).style(Style::new().fg(item.tag.color()));
            let file_str = item.file.display().to_string();
            let file_display = if file_str.len() > 30 {
                let chars: Vec<char> = file_str.chars().collect();
                let suffix: String = chars[chars.len().saturating_sub(28)..].iter().collect();
                format!("..{suffix}")
            } else {
                file_str
            };
            Row::new(vec![
                tag_cell,
                Cell::from(file_display),
                Cell::from(item.line.to_string()),
                Cell::from(item.author.as_deref().unwrap_or("-").to_string()),
                Cell::from(item.message.clone()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(30),
        Constraint::Length(6),
        Constraint::Length(12),
        Constraint::Fill(1),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Results ({}) ", app.filtered_items.len())),
        )
        .row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    let mut state = TableState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(item) = app.selected_item() else {
        let msg = Paragraph::new("  No item selected.")
            .style(Style::new().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Detail "));
        frame.render_widget(msg, area);
        return;
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("Tag:     ", Style::new().bold()),
            Span::styled(
                item.tag.to_string(),
                Style::new().fg(item.tag.color()).bold(),
            ),
        ]),
        Line::from(vec![
            Span::styled("File:    ", Style::new().bold()),
            Span::raw(item.file.display().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Line:    ", Style::new().bold()),
            Span::raw(item.line.to_string()),
        ]),
        Line::from(vec![
            Span::styled("Author:  ", Style::new().bold()),
            Span::raw(item.author.as_deref().unwrap_or("-")),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Message: ", Style::new().bold()),
            Span::raw(&item.message),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Raw:     ", Style::new().fg(Color::DarkGray)),
            Span::styled(&item.raw_line, Style::new().fg(Color::DarkGray)),
        ]),
    ];

    let detail = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Detail ")
            .border_style(Style::new().fg(item.tag.color())),
    );

    frame.render_widget(detail, area);
}

fn draw_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    let hints = match app.mode {
        AppMode::Normal => {
            "j/k: navigate  Enter: detail  /: search  1-8: filter  c: clear  q: quit"
        }
        AppMode::Search => "Type to search  Enter/Esc: exit search  Backspace: delete",
        AppMode::Detail => "Esc/Enter: back  j/k: prev/next  q: quit",
    };

    let line = Line::from(Span::styled(
        format!(" {hints}"),
        Style::new().fg(Color::DarkGray),
    ));
    frame.render_widget(Paragraph::new(line), area);
}
