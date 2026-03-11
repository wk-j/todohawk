use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::types::Tag;

use super::{App, AppMode};

pub enum Action {
    Quit,
    Continue,
}

pub fn handle_event(app: &mut App) -> anyhow::Result<Action> {
    if let Event::Key(key) = event::read()? {
        if key.kind != KeyEventKind::Press {
            return Ok(Action::Continue);
        }
        match app.mode {
            AppMode::Normal => return handle_normal(app, key.code),
            AppMode::Search => handle_search(app, key.code),
            AppMode::Detail => return handle_detail(app, key.code),
        }
    }
    Ok(Action::Continue)
}

fn handle_normal(app: &mut App, code: KeyCode) -> anyhow::Result<Action> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Enter => app.mode = AppMode::Detail,
        KeyCode::Char('/') => {
            app.mode = AppMode::Search;
        }
        KeyCode::Char('c') => {
            app.active_tag_filters.clear();
            app.search_query.clear();
            app.apply_filters();
        }
        KeyCode::Char(c @ '1'..='8') => {
            if let Some(tag) = tag_from_key(c) {
                app.toggle_tag_filter(tag);
            }
        }
        _ => {}
    }
    Ok(Action::Continue)
}

fn handle_search(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.apply_filters();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.apply_filters();
        }
        _ => {}
    }
}

fn handle_detail(app: &mut App, code: KeyCode) -> anyhow::Result<Action> {
    match code {
        KeyCode::Esc | KeyCode::Enter => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('q') => return Ok(Action::Quit),
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        _ => {}
    }
    Ok(Action::Continue)
}

fn tag_from_key(c: char) -> Option<Tag> {
    match c {
        '1' => Some(Tag::Todo),
        '2' => Some(Tag::Fixme),
        '3' => Some(Tag::Hack),
        '4' => Some(Tag::Xxx),
        '5' => Some(Tag::Note),
        '6' => Some(Tag::Optimize),
        '7' => Some(Tag::Bug),
        '8' => Some(Tag::Warn),
        _ => None,
    }
}
