use std::{io::{self, stdout}, ops::Add};
use crossterm::{
	event::{self, Event, KeyCode},
	ExecutableCommand,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}
};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use std::thread::sleep;



struct Player {
	score: u16,
	x: u16,
	y: u16,
	width: u16,
	height: u16,
}

struct PongBall {
	x: i32,
	y: i32,
	width: u16,
	height: u16,
	direction_x: i32,
	direction_y: i32,
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
			pongball: PongBall::new(t_width, t_height),
		}
	}
}

// Updated struct for your JSON data
#[derive(Debug, Serialize, Deserialize)]
struct GameStateSend {
    P_position_x: u16,
    P_position_y: u16,
    P_score: u16,
    screen_size_x: u16,
    screen_size_y: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameStateRecieve {
    P_position_x: u16,
    P_position_y: u16,
    Pongball_position_x: u16,
    Pongball_position_y: u16,
    P_score: u16,
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

			//Player Controls
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

		if (gameData.pongball.x == 4 && (gameData.pongball.y >= gameData.player.y as i32 && gameData.pongball.y <= (gameData.player.y + gameData.player.height) as i32 )) {
			gameData.pongball.direction_x *= -1;
		}
		if (gameData.pongball.x == t_size.width as i32 - 4 && (gameData.pongball.y >= gameData.opponent.y as i32 && gameData.pongball.y <= (gameData.opponent.y + gameData.opponent.height) as i32)) {
			gameData.pongball.direction_x *= -1;
		}

		if (gameData.pongball.x == 0) {
			gameData.opponent.score += 1;
			gameData.pongball.x = t_size.width as i32 /2;
			gameData.pongball.y = t_size.height as i32 /2;
		}
		if (gameData.pongball.x == t_size.width as i32 - 1) {
			gameData.player.score += 1;
			gameData.pongball.x = t_size.width as i32 /2;
			gameData.pongball.y = t_size.height as i32 /2; 
		}
	}
}

fn handle_server_recieve(stream: &mut TcpStream, gameDataT: &Arc<Mutex<GameData>>) {
	let send_interval = Duration::from_micros(1_000_000 / 128); // 1 second / 128
    let mut last_send_time = Instant::now();

    loop {

        // Buffer to store incoming data
        let mut buffer = [0; 1024];

        // Read data from the client
        let bytes_read = stream.read(&mut buffer).expect("Failed to read from client");

        // Deserialize the JSON data
        let received_data: GameStateRecieve = serde_json::from_slice(&buffer[..bytes_read])
            .expect("Failed to deserialize JSON data");

        // Process the received data
        println!("Received data: {:?}", received_data);
        
		let mut game = gameDataT.lock().unwrap();
		game.opponent.x = received_data.P_position_x;
		game.opponent.y = received_data.P_position_y;
		game.opponent.score = received_data.P_score;
		game.pongball.x = received_data.Pongball_position_x as i32;
		game.pongball.y = received_data.Pongball_position_y as i32;

        // Calculate the elapsed time since the last send
        let elapsed = last_send_time.elapsed();

        // If less time has passed than the desired interval, sleep to reach the interval
        if elapsed < send_interval {
            sleep(send_interval - elapsed);
        }

        // Update the last send time for the next iteration
        last_send_time = Instant::now();
    }
}

fn run() -> io::Result<()> {
	enable_raw_mode()?;
	stdout().execute(EnterAlternateScreen)?;
	let mut terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(CrosstermBackend::new(stdout()))?;
	//Save frame dimentions
	let mut currentFrameSize: Rect = terminal.get_frame().size();

	// let mut gameDataT: Arc<Mutex<GameData>> = Arc::new(Mutex::new(GameData::new(terminal.get_frame().size().width, terminal.get_frame().size().height)));
	let mut gameData = GameData::new(terminal.get_frame().size().width, terminal.get_frame().size().height);
	let mut stream = TcpStream::connect("127.0.0.1:25565").unwrap(); // Replace with the server address and port
	
	// let gameData = Arc::clone(&gameDataT);

	// let mut streamclone = stream.try_clone().expect("failed to clone");
	// std::thread::spawn(move || {
	// 	handle_server_recieve( &mut streamclone, &gameData);
	// });

	let mut should_quit = false;
	while !should_quit {
		// let mut gameData = gameDataT.lock().unwrap();

		let mut buffer = [0; 1024];

        // Read data from the client
        let bytes_read = stream.read(&mut buffer).expect("Failed to read from client");

        // Deserialize the JSON data
        let received_data: GameStateRecieve = serde_json::from_slice(&buffer[..bytes_read])
            .expect("Failed to deserialize JSON data");

        // Process the received data
        println!("Received data: {:?}", received_data);
        
		// let mut game = gameDataT.lock().unwrap();
		gameData.opponent.x = received_data.P_position_x;
		gameData.opponent.y = received_data.P_position_y;
		gameData.opponent.score = received_data.P_score;
		gameData.pongball.x = received_data.Pongball_position_x as i32;
		gameData.pongball.y = received_data.Pongball_position_y as i32;

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
			frame.render_widget(pong, Rect {	 x: gameData.pongball.x as u16,
															y: gameData.pongball.y as u16,
															width: 1,
															height: 1});
		})?;
		
		// Pong Controls
		// pong_controls(&mut gameData, terminal.get_frame().size());

		let mut sendData: GameStateSend = GameStateSend{P_position_x: gameData.player.x,
														P_position_y: gameData.player.y,
														P_score: gameData.player.score,
														screen_size_x: terminal.get_frame().size().width,
														screen_size_y: terminal.get_frame().size().height};
		let response_json = serde_json::to_vec(&sendData).expect("Failed to serialize JSON data");
		
		stream.write_all(&response_json).expect("Failed to write to client");

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
