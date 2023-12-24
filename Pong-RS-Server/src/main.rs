use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::thread::sleep;
use std::sync::{Arc, Mutex};


// Updated struct for your JSON data
#[derive(Debug, Serialize, Deserialize)]
struct GameStateRecieve {
    P_position_x: u16,
    P_position_y: u16,
    P_score: u16,
    screen_size_x: u16,
    screen_size_y: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct GameStateSend {
    P_position_x: u16,
    P_position_y: u16,
    Pongball_position_x: u16,
    Pongball_position_y: u16,
    P_score: u16,
}

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

struct Rect {
    height: u16,
    width: u16,
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

// Function to handle a single client
fn handle_client(mut stream: TcpStream, p: u8, game_state: &Arc<Mutex<GameData>>) {

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

        if (p == 0) {
            let mut game = game_state.lock().unwrap();
            game.player.x = received_data.P_position_x;
            game.player.y = received_data.P_position_y;
            game.player.score = received_data.P_score;
        }
        else {
            let mut game = game_state.lock().unwrap();
            game.opponent.x = received_data.P_position_x;
            game.opponent.y = received_data.P_position_y;
            game.opponent.score = received_data.P_score;
        }

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

fn pong_controls(gameDataCon: &Arc<Mutex<GameData>>, t_size: Rect) {
	//Pong Controls
    let mut gameData = gameDataCon.lock().unwrap();
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


fn main() {
    // Specify the server address
    let server_address = "127.0.0.1:25565";

    // Create a TcpListener
    let listener: TcpListener = TcpListener::bind(server_address).expect("Failed to bind to address");

    println!("Server listening on {}", server_address);

    let game_state = Arc::new(Mutex::new(GameData::new(35,64)));

    let mut active_connections: Vec<TcpStream> = Vec::new();
    // Accept incoming connections and spawn a new thread for each one

    let mut i: u8 = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {

                active_connections.push(stream.try_clone().expect("Failed to clone stream"));

                let game_state = Arc::clone(&game_state);

                std::thread::spawn(move || {
                    handle_client(stream, i, &game_state);
                });
                i += 1;
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    // Infinite loop for sending (128 times per second)
    let send_interval = Duration::from_micros(1_000_000 / 128); // 1 second / 128
    let mut last_send_time = Instant::now();
    
    loop {
        // Your sending logic goes here
        let mut i = 0;
        for connection in &active_connections {
            let mut stream = connection.try_clone().expect("failed to clone");

            
            // calculate ball position
            pong_controls(&game_state, Rect{height: 35, width: 64});

            let response_data = game_state.lock().unwrap();

            if (i == 1) {
                let gameStateSend = GameStateSend{
                    P_position_x: response_data.player.x,
                    P_position_y: response_data.player.y,
                    Pongball_position_x: response_data.pongball.x as u16,
                    Pongball_position_y: response_data.pongball.y as u16,
                    P_score: response_data.player.score,
                };
                // Serialize the response data to JSON
                let response_json = serde_json::to_vec(&gameStateSend).expect("Failed to serialize JSON data");

                // Send the JSON response back to the client
                stream.write_all(&response_json).expect("Failed to write to client");
            }
            else {
                let gameStateSend = GameStateSend{
                    P_position_x: response_data.opponent.x,
                    P_position_y: response_data.opponent.y,
                    Pongball_position_x: response_data.pongball.x as u16,
                    Pongball_position_y: response_data.pongball.y as u16,
                    P_score: response_data.opponent.score,
                };
                // Serialize the response data to JSON
                let response_json = serde_json::to_vec(&gameStateSend).expect("Failed to serialize JSON data");

                // Send the JSON response back to the client
                stream.write_all(&response_json).expect("Failed to write to client");
            }
            i += 1;
        }

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
