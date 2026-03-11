use std::io::{self, stdout};

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::types::TodoItem;

use super::App;
use super::event::{Action, handle_event};
use super::ui::draw;

pub fn run_tui(items: Vec<TodoItem>) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let result = run_app(items);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

fn run_app(items: Vec<TodoItem>) -> Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new(items);

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        match handle_event(&mut app)? {
            Action::Quit => break,
            Action::Continue => {}
        }
    }

    Ok(())
}
