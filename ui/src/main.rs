mod app;
mod form;
mod ui;

use std::io;

use anyhow::Result;
use app::{App, Field, Screen};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.screen {
                Screen::Form => handle_form_input(&mut app, key.code, key.modifiers),
                Screen::Preview => handle_preview_input(&mut app, key.code),
                Screen::Submitted => {
                    if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
                        app.should_quit = true;
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_form_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::Tab => app.focused = app.focused.next(),
        KeyCode::BackTab => app.focused = app.focused.prev(),
        KeyCode::Enter => app.submit_form(),
        KeyCode::Backspace => app.backspace(),
        KeyCode::Left => {
            if app.focused == Field::Currency {
                app.cycle_currency(false);
            }
        }
        KeyCode::Right => {
            if app.focused == Field::Currency {
                app.cycle_currency(true);
            }
        }
        KeyCode::Char(c) => {
            // Ctrl+C to quit
            if c == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                app.should_quit = true;
            } else {
                app.type_char(c);
            }
        }
        _ => {}
    }
}

fn handle_preview_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.confirm(),
        KeyCode::Esc => app.back(),
        _ => {}
    }
}
