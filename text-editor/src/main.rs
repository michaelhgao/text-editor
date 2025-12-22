mod models;

use crate::models::{document::Document, editor::Editor};

use std::{env, io};

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use tui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{Paragraph, Wrap},
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

            let lines: Vec<String> = editor
                .document()
                .lines()
                .iter()
                .map(|gb| gb.to_string())
                .collect();

            let text = lines.join("\n");

            let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });

            f.render_widget(paragraph, last_size);

            let (y, x) = editor.doc_to_screen(&last_size);
            f.set_cursor(x, y);
        })?;

        if let Event::Key(key_event) = event::read()? {
            editor.handle_key(key_event, &last_size);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
