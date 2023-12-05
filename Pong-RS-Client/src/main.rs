use std::io::{self, stdout};
use crossterm::{
    event::{self, Event, KeyCode},
    ExecutableCommand,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{prelude::*, widgets::*};

// player data
struct Player {
    score: usize,
    x: usize,
    y: usize,
    size: usize,
}

struct PongBall {
    x: usize,
    y: usize,
    prev_x: usize,
    prev_y: usize,
}

impl PongBall {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            prev_x: 0,
            prev_y: 0,
        }
    }
}

impl Player {
    fn new(x: usize, y: usize, size: usize) -> Self {
        Self {
            score: 0,
            x: x,
            y: y,
            size: size,
        }
    }
}

// terminal screen
struct TerminalOutput {
    player: Player,
    opponent: Player,
    pongball: PongBall,
}

impl TerminalOutput {
    fn new() -> Self {
        Self {
            player: Player::new(0, 2, 1),
            opponent: Player::new(0, 0, 0),
            pongball: PongBall::new(),
        }
    }

    fn run(&self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let mut should_quit = false;
        while !should_quit {
            terminal.draw(ui)?;
            should_quit = self.handle_events()?;
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_events(&self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    return Ok(true);
                }
                // if key.modifiers
           }
        }
        Ok(false)
    }
}

fn ui(frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("Hello World!")
            .block(Block::default().title("Greeting").borders(Borders::ALL)),
        frame.size(),
    );
}

fn main() -> io::Result<()>{
    let mut term = TerminalOutput::new();
    term.run()?;
    Ok(())
}
