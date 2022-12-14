use anyhow::Result;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use tui::backend::CrosstermBackend;

pub struct App {}

async fn _start_ui(_app: &Arc<Mutex<App>>) -> Result<()> {
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;

    let _backend = CrosstermBackend::new(stdout);

    /*
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    loop {
        let app = app.borrow();
        // Render
        terminal.draw(|rect| ui::draw(rect, &app))?;
        // TODO handle inputs here
    }

    // Restore the terminal and close application
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    */

    Ok(())
}
