pub mod model;
pub(crate) mod prelude;
pub(crate) mod shutils;
pub mod window;
pub(crate) mod workspace;
pub mod x11window;

use prelude::*;

use std::io::stdout;

use model::{Model, RunningState};
use prelude::Result;
use std::io::Stdout;

/// A type alias for the terminal type used in this application
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal
pub fn init_terminal() -> Result<Tui> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

/// Restore the terminal to its original state
pub fn restore_terminal() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<()> {
    let mut terminal = init_terminal()?;
    let frame = terminal.get_frame();
    let mut model = Model::new(&frame);

    while model.running_state != RunningState::Done {
        // Render the current view
        let _ = terminal.draw(|f| model.view(f).unwrap());

        // Handle events and map to a Message
        let mut current_msg = model.handle_event()?;

        // Process updates as long as they return a non-None message
        while current_msg.is_some() {
            current_msg = model.update(current_msg.unwrap());
        }
    }

    restore_terminal()?;
    Ok(())
}
