mod models;

use crate::models::editor::Editor;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use tui::Terminal;
use tui::backend::CrosstermBackend;

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
            f.render_widget(tui::widgets::Paragraph::new(text), f.size());

            let cursor = editor.cursor();
            f.set_cursor(cursor.1 as u16, cursor.0 as u16);
        })?;

        if let Event::Key(key_event) = event::read()? {
            editor.process_keypress(key_event);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
