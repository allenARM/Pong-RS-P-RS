use std::io::{self, stdout};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::{terminal, ExecutableCommand};

// player data
struct Player {}

impl Player {
    fn new() -> Self {
        Self {}
    }
}

// terminal screen
struct TerminalOutput {
    player: Player,
    opponent: Player,
}

impl TerminalOutput {
    fn new(player: &mut Player, opponent: &mut Player) -> Self {
        Self {
            player: Player::new(),
            opponent: Player::new(),
        }
    }

    fn run(&self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        stdout().execute(terminal::EnterAlternateScreen)?;

        while self.read_key(event::read()?)? {
            // do some processing 
        }

        terminal::disable_raw_mode()?;
        stdout().execute(terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    fn read_key(&self, e: Event) -> io::Result<bool> {
        match e {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            }) => return Ok(false),
            _ => {},
        } 
        Ok(true)
    }


}

fn main() -> io::Result<()>{
    let mut player = Player::new();
    let mut opponent = Player::new();
    let mut term = TerminalOutput::new(&mut player, &mut opponent);
    term.run()?;
    Ok(())
}
