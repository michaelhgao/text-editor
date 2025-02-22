mod models;

use crate::models::editor::Editor;

use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::CrosstermBackend;
use tui::Terminal;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut editor = Editor::new();

    while !editor.should_quit() {
        terminal.draw(|f| {
            let text = editor.document().lines().join("\n");
            f.render_widget(tui::widgets::Paragraph::new(text), f.size());
        })?;

        if let Event::Key(key_event) = event::read()? {
            editor.process_keypress(key_event);
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;    
    Ok(())
}
