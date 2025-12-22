mod models;

use crate::models::{
    document::Document,
    editor::{Editor, Mode},
};

use std::{env, io};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use tui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Wrap},
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <file>", args[0]);
        return Ok(());
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let doc = if let Some(path) = args.get(1) {
        match Document::open(path) {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Failed to open '{}': {}", path, e);
                Document::new()
            }
        }
    } else {
        Document::new()
    };
    let mut editor = Editor::new(doc);

    while !editor.should_quit() {
        let mut last_size = terminal.size()?;
        terminal.draw(|f| {
            last_size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // editor area
                    Constraint::Length(1), // status bar
                ])
                .split(last_size);

            draw_editor(f, &chunks[0], &editor);
            draw_status_bar(f, &chunks[1], &editor);
        })?;

        if let Event::Key(key_event) = event::read()? {
            editor.handle_key(key_event, &last_size);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw_editor<B: Backend>(f: &mut Frame<B>, area: &Rect, editor: &Editor) {
    let lines: Vec<String> = editor
        .doc()
        .lines()
        .iter()
        .map(|gb| gb.to_string())
        .collect();

    let text = lines.join("\n");

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });

    f.render_widget(paragraph, *area);

    let (y, x) = editor.doc_to_screen(&area);
    f.set_cursor(x, y);
}

fn draw_status_bar<B: Backend>(f: &mut Frame<B>, area: &Rect, editor: &Editor) {
    let status = match editor.mode() {
        Mode::Command => format!(":{}", editor.command_buffer()),
        _ => format!(
            " {} | {} | {}:{} ",
            editor.mode().as_str(),
            editor.doc().file_name(),
            editor.cursor().0 + 1,
            editor.cursor().1 + 1
        ),
    };

    let paragraph = Paragraph::new(status)
        .style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default());

    f.render_widget(paragraph, *area);
}
