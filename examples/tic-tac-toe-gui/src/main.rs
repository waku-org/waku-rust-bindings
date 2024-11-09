use eframe::egui;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::str::from_utf8;
use std::time::SystemTime;
use std::cell::OnceCell;
use waku::{
    waku_new, Event, WakuNodeConfig,
    LibwakuResponse, Multiaddr, Running, WakuNodeHandle,
    Initialized, WakuContentTopic, WakuMessage, Encoding,
    WakuNodeContext,
};

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
enum Player {
    X,
    O,
}

#[derive(Serialize, Deserialize, Clone)]
struct GameState {
    board: [[Option<Player>; 3]; 3],
    current_turn: Player,
    moves_left: usize,
}

#[derive(Clone)]
struct MoveMessage {
    row: usize,
    col: usize,
    player: Player,
}

struct TicTacToeApp {
    game_state: Arc<Mutex<GameState>>,
    val: Arc<Mutex<String>>,
    waku: WakuNodeHandle,
    game_topic: &'static str,
    tx: Arc<Mutex<mpsc::Sender<String>>>, // Sender to send `msg` to main thread
}

impl TicTacToeApp {
    fn new(waku: WakuNodeHandle,
           game_topic: &'static str,
           tx: Arc<Mutex<mpsc::Sender<String>>>,) -> Self {
        Self {
            game_state: Arc::new(Mutex::new(GameState {
                board: [[None; 3]; 3],
                current_turn: Player::X,
                moves_left: 9,
            })),
            val: Arc::new(Mutex::new("".to_string())),
            waku,
            game_topic,
            tx: tx,
        }
    }

    fn start(&mut self) {
        // Start the waku node
        self.waku.start().expect("waku should start");

        // let default_pubsub_topic = Arc::new(Mutex::new("".to_string()));
        // let shared_data_clone = Arc::clone(&default_pubsub_topic);
        // // Establish a closure that handles the incoming messages
        // self.waku.ctx.waku_set_event_callback(|response| {

        //     let mut data = shared_data_clone.lock().unwrap();
        //     *data = "Updated from another thread".to_string(); // Write access
            
        //     println!("funciona ?");
        
        
     //   let mut cloned = Arc::clone(&self.val);

        let tx_clone = self.tx.clone();
        // Establish a closure that handles the incoming messages
        self.waku.ctx.waku_set_event_callback(move |response| {
            //   if let Ok(mut tx) = tx_clone.try_lock() {
            //                       //  Lock succeeded, proceed to send the message
            //                         if tx.try_send(msg.to_string()).is_err() {
            //                             eprintln!("Failed to send message to async task");
            //                         }
            //                     } else {
            //                         eprintln!("Failed to acquire lock on tx_clone");
            //                     }
        //    if let Ok(mut aa) = cloned.try_lock() {

        //     }
            
        //     match cloned.lock() {
        //         Ok(mut data) => {
        //             *data = "Modified Value".to_string();
        //             println!("Thread updated value to: {}", data);
        //         },
        //         Err(e) => {
        //             eprintln!("Failed to lock the mutex in thread: {}", e);
        //         }
        //     }


            if let LibwakuResponse::Success(v) = response {
                let event: Event =
                    serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

                // let mut game_state = self.game_state.lock().unwrap();
                match event {
                    Event::WakuMessage(evt) => {
                        // println!("WakuMessage event received: {:?}", evt.waku_message);
                        let message = evt.waku_message;
                        let payload = message.payload.to_vec().clone();
                        match from_utf8(&payload) {
                            Ok(msg) => {
                                println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                                println!("Message Received: {}", msg);
                                println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");

                                // Send the message to the main thread
                                if let Ok(mut tx) = tx_clone.try_lock() {
                                  //  Lock succeeded, proceed to send the message
                                    if tx.try_send(msg.to_string()).is_err() {
                                        eprintln!("Failed to send message to async task");
                                    }
                                } else {
                                    eprintln!("Failed to acquire lock on tx_clone");
                                }

                                // Deserialize the JSON into the GameState struct
                                // Lock the game_state and update it
                                // match serde_json::from_str::<GameState>(msg) {
                                //     Ok(parsed_value) => {
                                //         // Handle the parsed value here
                                //         // self.game_state = parsed_value;
                                //         println!("Parsed correctly");
                                //     }
                                //     Err(e) => {
                                //         eprintln!("Failed to parse JSON: {}", e);
                                //         // Handle the error as needed, such as retrying, defaulting, etc.
                                //     }
                                // }
                                // *game_state = serde_json::from_str(msg).expect("Failed to deserialize JSON");

                                // let tx_inner = tx_cloned.clone();
                                // let msg_inner = msg.to_string();
                                // tokio::spawn(async move {
                                //     println!("do nothing");
                                // if tx_inner.send(msg_inner.to_string()).await.is_err() {
                                //     eprintln!("Failed to send message");
                                // }
                                // });
                            }
                            Err(e) => {
                                eprintln!("Failed to decode payload as UTF-8: {}", e);
                                // Handle the error as needed, or just log and skip
                            }
                        }
                    }
                    Event::Unrecognized(err) => panic!("Unrecognized waku event: {:?}", err),
                    _ => panic!("event case not expected"),
                };
            }
        });
        
        // Subscribe to desired topic
        self.waku.relay_subscribe(&self.game_topic.to_string()).expect("waku should subscribe");
        
        // Connect to hard-coded node
        let target_node_multi_addr =
            "/ip4/24.144.78.119/tcp/30303/p2p/16Uiu2HAm3xVDaz6SRJ6kErwC21zBJEZjavVXg7VSkoWzaV1aMA3F"
            .parse::<Multiaddr>().expect("parse multiaddress");

        self.waku.connect(&target_node_multi_addr, None)
             .expect("waku should connect to other node");
    }

    fn send_game_state(&self, game_state: &GameState) {

        let serialized_game_state = serde_json::to_string(game_state).unwrap();
        let content_topic = WakuContentTopic::new("waku", "2", "tictactoegame", Encoding::Proto);

        let message = WakuMessage::new(
            &serialized_game_state,
            content_topic,
            0,
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .unwrap(),
            Vec::new(),
            false,
        );

        // let waku_handle = self.waku.lock().unwrap();
        self.waku.relay_publish_message(&message, &self.game_topic.to_string(), None)
            .expect("Failed to send message");
    }

    fn make_move(&mut self, row: usize, col: usize) {
        if let Ok(mut game_state) = self.game_state.try_lock() {
            if (*game_state).board[row][col].is_none() && (*game_state).moves_left > 0 {
                (*game_state).board[row][col] = Some((*game_state).current_turn);
                (*game_state).moves_left -= 1;

                if let Some(winner) = self.check_winner(&game_state) {
                    (*game_state).current_turn = winner;
                } else {
                    (*game_state).current_turn = match (*game_state).current_turn {
                        Player::X => Player::O,
                        Player::O => Player::X,
                    };
                }

                self.send_game_state(&game_state); // Send updated state after a move
            }
        }
    }

    fn check_winner(&self, game_state: &GameState) -> Option<Player> {
        // Check rows, columns, and diagonals
        for i in 0..3 {
            if game_state.board[i][0] == game_state.board[i][1] && 
               game_state.board[i][1] == game_state.board[i][2] {
                if let Some(player) = game_state.board[i][0] {
                    return Some(player);
                }
            }
            if game_state.board[0][i] == game_state.board[1][i] &&
               game_state.board[1][i] == game_state.board[2][i] {
                if let Some(player) = game_state.board[0][i] {
                    return Some(player);
                }
            }
        }
        if game_state.board[0][0] == game_state.board[1][1] &&
           game_state.board[1][1] == game_state.board[2][2] {
            if let Some(player) = game_state.board[0][0] {
                return Some(player);
            }
        }
        if game_state.board[0][2] == game_state.board[1][1] &&
           game_state.board[1][1] == game_state.board[2][0] {
            if let Some(player) = game_state.board[0][2] {
                return Some(player);
            }
        }
        None
    }

    fn reset_game(&mut self) {
        self.game_state = Arc::new(Mutex::new(GameState {
            board: [[None; 3]; 3],
            current_turn: Player::X,
            moves_left: 9,
        }));
    }
}

impl eframe::App for TicTacToeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tic-Tac-Toe");

            let text_size = 32.0;
            let board_size = ui.available_size();
            let cell_size = board_size.x / 4.0;

            ui.horizontal(|ui| {
                for row in 0..3 {
                    ui.vertical(|ui| {
                        for col in 0..3 {
                            let label;
                            {
                                if let Ok(game_state) = self.game_state.try_lock() {
                                    label = match game_state.board[row][col] {
                                        Some(Player::X) => "X",
                                        Some(Player::O) => "O",
                                        None => "-",
                                    };
                                }
                                else {
                                    label = "#";
                                }
                            }

                            let button = ui.add(egui::Button::new(label).min_size(egui::vec2(cell_size, cell_size)).sense(egui::Sense::click()));

                            if button.clicked() {
                                self.make_move(row, col);
                            }
                        }
                    });
                    if row < 2 {
                        ui.add_space(4.0);
                    }
                }
            });

            if let Ok(game_state) = self.game_state.try_lock() {
                if let Some(winner) = self.check_winner(&game_state) {
                    ui.label(format!(
                        "Player {} wins!",
                        match winner {
                            Player::X => "X",
                            Player::O => "O",
                        }
                    ));
                } else if game_state.moves_left == 0 {
                    ui.label("It's a tie!");
                } else {
                    ui.label(format!(
                        "Player {}'s turn",
                        match game_state.current_turn {
                            Player::X => "X",
                            Player::O => "O",
                        }
                    ));
                }
            }

            if ui.add(egui::Button::new("Restart Game")).clicked() {
                self.reset_game();
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let (tx, mut rx) = mpsc::channel::<String>(3200); // Channel to communicate between threads

    // Create a Waku instance
    let waku = waku_new(Some(WakuNodeConfig {
        port: Some(60010),
        cluster_id: Some(16),
        log_level: Some("DEBUG"), // Supported: TRACE, DEBUG, INFO, NOTICE, WARN, ERROR or FATAL
        ..Default::default()
    }))
    .expect("should instantiate");
    // Initialize Waku
    let game_topic = "/waku/2/rs/16/64";
    let mut app = TicTacToeApp::new(waku, game_topic, Arc::new(Mutex::new(tx)));

    app.start();

    eframe::run_native(
        "Tic-Tac-Toe Multiplayer via Waku",
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(400.0, 400.0)),
            ..Default::default()
        },
        Box::new(|_cc| Box::new(app)),
    )?;

    // Listen for messages in the main thread
    tokio::spawn(async move {
        unsafe {
            waku_sys::waku_setup();
        }
        while let Some(msg) = rx.recv().await {
            println!("Main thread received: {}", msg);
            // Handle the received message, e.g., update the UI or game state
        }
    });

    Ok(())
}
