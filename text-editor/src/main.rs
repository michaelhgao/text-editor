mod models;

use crate::models::{document::Document, editor::Editor};

use std::io;

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

fn doc_to_screen(doc: &Document, cursor: (usize, usize), width: usize) -> (u16, u16) {
    let (row, col) = cursor;

    let mut screen_row = 0;

    for i in 0..row {
        let len = doc.lines()[i].len();
        screen_row += (len / width).max(1);
    }

    let wrapped_row = col / width;
    let wrapped_col = col % width;

    ((screen_row + wrapped_row) as u16, wrapped_col as u16)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut editor = Editor::new();

    while !editor.should_quit() {
        terminal.draw(|f| {
            let lines: Vec<String> = editor
                .document()
                .lines()
                .iter()
                .map(|gb| gb.to_string())
                .collect();
            let text = lines.join("\n");
            let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });
            let size = f.size();
            f.render_widget(paragraph, size);

            let (y, x) = doc_to_screen(editor.document(), editor.cursor(), size.width as usize);
            f.set_cursor(x, y);
        })?;

        if let Event::Key(key_event) = event::read()? {
            editor.process_keypress(key_event);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
