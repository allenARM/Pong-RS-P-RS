use std::{io::{self, stdout}, ops::Add};
use crossterm::{
	event::{self, Event, KeyCode},
	ExecutableCommand,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{prelude::*, widgets::*};

struct Player {
	score: u16,
	x: u16,
	y: u16,
	width: u16,
	height: u16,
}

struct PongBall {
	x: u16,
	y: u16,
	prev_x: u16,
	prev_y: u16,
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
	fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
		Self {
			score: 0,
			x: x,
			y: y,
			width: width,
			height: height,
		}
	}
}

// terminal screen
struct GameData {
	player: Player,
	opponent: Player,
	pongball: PongBall,
}

impl GameData {
	fn new(t_width: u16, t_height: u16) -> Self {
		Self {
			player: Player::new(2, t_height/2-4, 2, 8),
			opponent: Player::new(t_width-4, t_height/2-4, 2, 8),
			pongball: PongBall::new(),
		}
	}
}

fn handle_events(gameData: &mut GameData, t_size: Rect) -> io::Result<bool> {
	if event::poll(std::time::Duration::from_millis(30))? {
		if let Event::Key(key) = event::read()? {
			if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
				return Ok(true);
			}

			//Player Controls
			if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Up {
				if (gameData.player.y > 1) {
					gameData.player.y -= 1;
				}
			}
			if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Down {
				if (gameData.player.y < t_size.height-gameData.player.height-1) {
					gameData.player.y += 1;
				}
			}
	   }
	}
	Ok(false)
}

fn run() -> io::Result<()> {
	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(CrosstermBackend::new(stdout()))?;

	let mut gameData: GameData = GameData::new(terminal.get_frame().size().width, terminal.get_frame().size().height);


	let mut should_quit = false;
	while !should_quit {
		terminal.draw( | frame | {
			//Draw title and borders around
			let x = frame.size().width.to_string().add("x").add(frame.size().height.to_string().as_str());
			frame.render_widget(Block::default().title_alignment(Alignment::Center).title(	" P1: ".to_string() +
																									&gameData.player.score.to_string() +
																									&" ---Welcome to Pong-RS/P-RS--- ".to_string() +
																									&x +
																									&" P2: ".to_string() +
																									&gameData.opponent.score.to_string() +
																									&" ".to_string()).borders(Borders::ALL), frame.size());			


			//Draw first player
			let p1 = Block::default().borders(Borders::ALL);
			frame.render_widget(p1, Rect { x: gameData.player.x as u16, 
														y: gameData.player.y as u16,
														width: gameData.player.width as u16,
														height: gameData.player.height as u16 });

			//Draw second player
			let p2 = Block::default().borders(Borders::ALL);
			frame.render_widget(p2, Rect { x: gameData.opponent.x as u16,
														y: gameData.opponent.y as u16,
														width: gameData.opponent.width as u16,
														height: gameData.opponent.height as u16 });
		})?;
		should_quit = handle_events(&mut gameData, terminal.get_frame().size())?;
	}

	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;
	Ok(())
}

fn main() -> io::Result<()>{
	run()?;
	Ok(())
}
