use std::{io::{self, stdout}, ops::Add, time::Duration};
use crossterm::{
	event::{self, Event, KeyCode},
	ExecutableCommand,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{prelude::*, widgets::*};
use std::env;
use std::net::TcpListener;
use std::net::TcpStream;
// use std::sync::mpsc;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::io::{ErrorKind, Read, Write};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Player {
	score: u16,
	x: u16,
	y: u16,
	width: u16,
	height: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PongBall {
	x: i32,
	y: i32,
	width: u16,
	height: u16,
	direction_x: i32,
	direction_y: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct sendData {
	oponent: Player,
	pongball: PongBall
}

impl PongBall {
	fn new(t_width: u16, t_height: u16) -> Self {
		Self {
			x: t_width as i32/2,
			y: t_height as i32/2,
			width: 1,
			height: 1,
			direction_x: 1,
			direction_y: 1,
		}
	}

	fn clone(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            direction_x: self.direction_x,
            direction_y: self.direction_y,
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

	fn clone(&self) -> Self {
        Self {
            score: self.score,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
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
			pongball: PongBall::new(t_width, t_height),
		}
	}
	fn clone(&self) -> Self {
        Self {
            player: self.player.clone(),
            opponent: self.opponent.clone(),
            pongball: self.pongball.clone(),
        }
    }
}

fn handle_events(gameData: &mut GameData, t_size: Rect, server: i32) -> io::Result<bool> {
	if event::poll(std::time::Duration::from_millis(30))? {
		if let Event::Key(key) = event::read()? {
			if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
				return Ok(true);
			}

			//Player/Client Controls
			if (server == 1) {
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

			//Player/Server Controls
			if (server == 0) {
				if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('w') {
					if (gameData.opponent.y > 1) {
						gameData.opponent.y -= 1;
					}
				}
				if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('s') {
					if (gameData.opponent.y < t_size.height-gameData.opponent.height-1) {
						gameData.opponent.y += 1;
					}
				}
			}
	   }
	}
	Ok(false)
}

fn handle_terminal_size_change(currentFrameSize: &mut Rect, gameData: &mut GameData, newTerminalSize: Rect) {
	if ((currentFrameSize.height != newTerminalSize.height) || (currentFrameSize.width != newTerminalSize.width)) {
		*currentFrameSize = newTerminalSize;
		//save scores of p1 and p2
		let score_p1: u16 = gameData.player.score;
		let score_p2: u16 = gameData.opponent.score;
		*gameData = GameData::new(currentFrameSize.width, currentFrameSize.height);
		gameData.player.score = score_p1;
		gameData.opponent.score = score_p2;
	}
}

fn pong_controls(gameData: &mut GameData, t_size: Rect) {
	//Pong Controls
	{
		gameData.pongball.x += gameData.pongball.direction_x;
		gameData.pongball.y += gameData.pongball.direction_y;

		//Checking Y directions
		if (gameData.pongball.y == 0 || gameData.pongball.y == t_size.height as i32 - 1) {
			gameData.pongball.direction_y *= -1;
		}

		//Checking if hit the paddles
		// if (gameData.pongball.x == 2 || gameData.pongball.x == t_size.width as i32 - 2) {
		// 	gameData.pongball.direction_x *= -1;
		// }

		if (gameData.pongball.x == 4 && (gameData.pongball.y >= gameData.player.y as i32 && gameData.pongball.y <= (gameData.player.y + gameData.player.height) as i32 )) {
			gameData.pongball.direction_x *= -1;
		}
		if (gameData.pongball.x == t_size.width as i32 - 4 && (gameData.pongball.y >= gameData.opponent.y as i32 && gameData.pongball.y <= (gameData.opponent.y + gameData.opponent.height) as i32)) {
			gameData.pongball.direction_x *= -1;
		}

		if (gameData.pongball.x == 0) {
			gameData.player.score += 1;
			gameData.pongball.x = t_size.width as i32 /2;
			gameData.pongball.y = t_size.height as i32 /2;
		}
		if (gameData.pongball.x == t_size.width as i32 - 1) {
			gameData.opponent.score += 1;
			gameData.pongball.x = t_size.width as i32 /2;
			gameData.pongball.y = t_size.height as i32 /2; 
		}
	}
}

fn runServer() -> io::Result<()> {
	let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
	server.set_nonblocking(true).expect("failed to initialize non-blocking");

	// let mut clients = vec![];
	let (tx, rx) = mpsc::channel::<String>();



	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(CrosstermBackend::new(stdout()))?;
	//Save frame dimentions
	let mut currentFrameSize: Rect = terminal.get_frame().size();

	let mut gameData: GameData = GameData::new(terminal.get_frame().size().width, terminal.get_frame().size().height);

	let mut should_quit = false;
	while !should_quit {
		//server
		if let Ok((mut socket, addr)) = server.accept() {
			println!("Client {} connected", addr);

			let tx = tx.clone();
			// clients.push(socket.try_clone().expect("failed to clone client"));

			let newGameData = gameData.clone();
			thread::spawn(move || loop {
				let mut buff = vec![0; MSG_SIZE];

				match socket.read_exact(&mut buff) {
					Ok(_) => {
						// let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
						// let msg = String::from_utf8(msg).expect("Invalid utf8 message");
						let dataToSend: sendData = sendData{ oponent: newGameData.opponent.clone(), pongball: newGameData.pongball.clone()};
						let msg = serde_json::to_string(&dataToSend).unwrap();
						// println!("{}: {:?}", addr, msg);
						tx.send(msg).expect("failed to send msg to rx");
					}, 
					Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
					Err(_) => {
						println!("closing connection with: {}", addr);
						break;
					}
				}
			});
		}

		if let Ok(msg) = rx.try_recv() {
			// let mut buff = msg.clone().into_bytes();
			// buff.resize(MSG_SIZE, 0);
			let deserializedMsg: Player = serde_json::from_str(&msg).unwrap();
			gameData.opponent = deserializedMsg;
		}


		//Check terminal size change
		handle_terminal_size_change(&mut currentFrameSize, &mut gameData, terminal.get_frame().size());

		terminal.draw( | frame | {
			//Draw title and borders around
			let x = frame.size().width.to_string().add("x").add(frame.size().height.to_string().as_str());
			frame.render_widget(Block::default().title_alignment(Alignment::Center).title(	" P1: ".to_string() +
																									&gameData.player.score.to_string() +
																									&" ---Welcome to Pong-RS/P-RS--- ".to_string() +
																									&x +
																									&" P2: ".to_string() +
																									&gameData.opponent.score.to_string() +
																									&" ".to_string()).borders(Borders::ALL).fg(Color::Cyan), frame.size());			


			//Draw first player
			let p1 = Block::default().borders(Borders::NONE).bg(Color::LightGreen);
			frame.render_widget(p1, Rect { x: gameData.player.x as u16, 
														y: gameData.player.y as u16,
														width: gameData.player.width as u16,
														height: gameData.player.height as u16 });

			//Draw second player
			let p2 = Block::default().borders(Borders::NONE).bg(Color::LightMagenta);
			frame.render_widget(p2, Rect { x: gameData.opponent.x as u16,
														y: gameData.opponent.y as u16,
														width: gameData.opponent.width as u16,
														height: gameData.opponent.height as u16 });

			//Draw pong
			let pong = Paragraph::new("o").alignment(Alignment::Center).fg(Color::White);
			// let pong = Block::default().borders(Borders::NONE).bg(Color::Red);
			frame.render_widget(pong, Rect {	 x: gameData.pongball.x as u16,
															y: gameData.pongball.y as u16,
															width: 1,
															height: 1});
		})?;
		// Pong Controls
		pong_controls(&mut gameData, terminal.get_frame().size());

		should_quit = handle_events(&mut gameData, terminal.get_frame().size(), 1)?;
	}

	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;
	Ok(())
}

fn runClient() -> io::Result<()> {
	
	let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
	client.set_nonblocking(true).expect("failed to initiate non-blocking");

	let (tx, rx) = mpsc::channel::<String>();

	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(CrosstermBackend::new(stdout()))?;
	//Save frame dimentions
	let mut currentFrameSize: Rect = terminal.get_frame().size();

	let mut gameData: GameData = GameData::new(terminal.get_frame().size().width, terminal.get_frame().size().height);

	let mut should_quit = false;
	while !should_quit {
		//connection to the server
		let mut client = client.try_clone().expect("failed clone");
		let gameDataClone = gameData.clone();
		// let txClone = tx.clone();
		thread::spawn(move || loop {
			let mut buff = vec![0; MSG_SIZE];
			let msg = serde_json::to_string(&gameDataClone.player).unwrap();
			// client.write_all(&serialized);
			// tx.send(msg).expect("failed to send msg to rx");
			client.write_all(&msg.as_bytes()).expect("failed to send");

		});
		if let Ok(msg) = rx.try_recv() {
			// let mut buff = msg.clone().into_bytes();
			// buff.resize(MSG_SIZE, 0);
			let deserializedMsg: sendData = serde_json::from_str(&msg).unwrap();
			gameData.opponent = deserializedMsg.oponent;
			gameData.pongball = deserializedMsg.pongball;
		}

		//Check terminal size change
		handle_terminal_size_change(&mut currentFrameSize, &mut gameData, terminal.get_frame().size());

		terminal.draw( | frame | {
			//Draw title and borders around
			let x = frame.size().width.to_string().add("x").add(frame.size().height.to_string().as_str());
			frame.render_widget(Block::default().title_alignment(Alignment::Center).title(	" P1: ".to_string() +
																									&gameData.player.score.to_string() +
																									&" ---Welcome to Pong-RS/P-RS--- ".to_string() +
																									&x +
																									&" P2: ".to_string() +
																									&gameData.opponent.score.to_string() +
																									&" ".to_string()).borders(Borders::ALL).fg(Color::Cyan), frame.size());			


			//Draw first player
			let p1 = Block::default().borders(Borders::NONE).bg(Color::LightGreen);
			frame.render_widget(p1, Rect { x: gameData.player.x as u16, 
														y: gameData.player.y as u16,
														width: gameData.player.width as u16,
														height: gameData.player.height as u16 });

			//Draw second player
			let p2 = Block::default().borders(Borders::NONE).bg(Color::LightMagenta);
			frame.render_widget(p2, Rect { x: gameData.opponent.x as u16,
														y: gameData.opponent.y as u16,
														width: gameData.opponent.width as u16,
														height: gameData.opponent.height as u16 });

			//Draw pong
			let pong = Paragraph::new("o").alignment(Alignment::Center).fg(Color::White);
			// let pong = Block::default().borders(Borders::NONE).bg(Color::Red);
			frame.render_widget(pong, Rect {	 x: gameData.pongball.x as u16,
															y: gameData.pongball.y as u16,
															width: 1,
															height: 1});
		})?;
		// Pong Controls
		// pong_controls(&mut gameData, terminal.get_frame().size());

		should_quit = handle_events(&mut gameData, terminal.get_frame().size(), 0)?;
	}

	disable_raw_mode()?;
	stdout().execute(LeaveAlternateScreen)?;
	Ok(())
}


const LOCAL: &str = "127.0.0.1:25565";
const DEFAULT_PORT: &str = "25565";
const MSG_SIZE: usize = 2048;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() -> io::Result<()> {
	let args: Vec<String> = env::args().collect();

	// default port 25565
	// program runs "cargo run ip port" to connect to the server
	// or "cargo run port" to start the server 

	//start the server
	if (args.len() == 1) {
		runServer();
	}
	if (args.len() == 2) {
		runClient();
	}
	Ok(())
}
