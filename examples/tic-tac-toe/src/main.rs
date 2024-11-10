extern crate termcolor;

use std::sync::mpsc; // For standard channels
use std::thread;
use tokio::sync::mpsc as tokio_mpsc; // For tokio's asynchronous channels
use tokio::io::{self, AsyncBufReadExt}; // For async I/O
use tokio::time::{sleep, Duration};
use tokio::task::LocalSet;
use tokio::runtime::Builder;
use tokio::sync::OnceCell;

use std::sync::{Arc, Mutex};

use std::io::Write;
use std::str::from_utf8;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use waku::{
    waku_new, Event, WakuNodeConfig,
    LibwakuResponse, Multiaddr, Running, WakuNodeHandle,
};

fn greeting() {
    println!(
        "\nRust TicTacToe\n\
         --------------\n\
         A simple game written in the rust programming language.\n\
         Credits to: https://github.com/flofriday/tictactoe"
    )
}

fn print_player(current_player: &char) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if current_player == &'X' {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Blue)))
            .unwrap();
    } else if current_player == &'O' {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
            .unwrap();
    }

    write!(&mut stdout, "{}", current_player).unwrap();
    stdout.reset().unwrap();
}

fn draw(board: &[char]) {
    println!("\n");

    for i in (0..3).rev() {
        let offset = i * 3;

        print!("-------------\n| ");
        print_player(&board[offset]);
        print!(" | ");
        print_player(&board[offset + 1]);
        print!(" | ");
        print_player(&board[offset + 2]);
        println!(" |");
    }

    println!("-------------");
}

fn ask_game_name_to_user(game_name: &mut String) {
    println!("Indicate the game name (arbitrary word)");

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        println!("Couldn't read line! Try again.");
    }

    if let Ok(val) = input.trim().parse::<String>() {
        *game_name = val;
    } else {
        println!("Only chars are allowed.");
    }
}

fn ask_role_to_user(role: &mut char) {
    println!("Select your role, X or O:");

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        println!("Couldn't read line! Try again.");
    }

    if let Ok(val) = input.trim().parse::<char>() {
        if val != 'X' && val != 'O' {
            println!("The user role must be either X or O.");
            return;
        }

        *role = val;
    } else {
        println!("Only chars are allowed.");
    }
}

fn ask_user(board: &mut [char], current_player: char) {
    print!("Player '");
    print_player(&current_player);
    println!("', enter a number: ");

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        println!("Couldn't read line! Try again.");
        return;
    }

    if let Ok(number) = input.trim().parse::<usize>() {
        if number < 1 || number > 9 {
            println!("The field number must be between 1 and 9.");
        }

        let number = number - 1;

        if board[number] == 'X' || board[number] == 'O' {
            print!("This field is already taken by '");
            print_player(&board[number]);
            println!("'.");
        }

        board[number] = current_player;
    } else {
        println!("Only numbers are allowed.");
    }
}

fn has_won(board: &[char]) -> bool {
    for tmp in 0..3 {
        if board[tmp] == board[tmp + 3] && board[tmp] == board[tmp + 6] {
            return true;
        }

        let tmp = tmp * 3;

        if board[tmp] == board[tmp + 1] && board[tmp] == board[tmp + 2] {
            return true;
        }
    }

    if (board[0] == board[4] && board[0] == board[8])
        || (board[2] == board[4] && board[2] == board[6])
    {
        return true;
    }

    false
}

#[inline(always)]
fn is_over(board: &[char]) -> bool {
    board.iter().all(|&v| v == 'X' || v == 'O')
}

// Return true if the game should end.
// false otherwise
fn game_logic(current_player: &mut char, board: &mut [char],
              topic: &str, waku: &WakuNodeHandle<Running>) -> bool {

    // Check if a player won
    if has_won(&board) {
        draw(&board);
        print!("Player '");
        print_player(&current_player);
        println!("' won! \\(^.^)/");
        return true;
    }

    // Check if all fields are used
    if is_over(&board) {
        draw(&board);
        println!("All fields are used. No one won. (._.)");
        return true;
    }

    return false;
}

#[tokio::main]
async fn main() {
    // Create a channel for communication
    let buffer_size = 256;
    // let (tx, mut rx) = tokio_mpsc::channel(buffer_size);

    let mut board = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let mut current_player = 'X';
    let mut my_role = 'X'; // Keeps track of my role, X or O.
    let mut game_name = "anonymous".to_string();
    let topic = "/waku/2/rs/16/64".to_string();

    // Create a Waku instance
    let waku = waku_new(Some(WakuNodeConfig {
        port: Some(60010),
        cluster_id: Some(16),
        log_level: Some("ERROR"), // Supported: TRACE, DEBUG, INFO, NOTICE, WARN, ERROR or FATAL
        ..Default::default()
    }))
    .expect("should instantiate");

    // ctrlc::set_handler(move ||{
    //     println!("Ctrl+C detected. Exiting gracefully...");
    //     // waku.stop();
    // }).expect("Error setting Ctrl+C handler");
    let waku = waku.start().expect("waku should start");
    // let tx_cloned = tx.clone();

    // Wait for tasks to complete
    let _ = tokio::join!(setter, reader);
    
    // Establish a closure that handles the incoming messages
    waku.set_event_callback(|response| {
        if let LibwakuResponse::Success(v) = response {
            let event: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

            match event {
                Event::WakuMessage(evt) => {
                    println!("WakuMessage event received: {:?}", evt.waku_message);
                    let message = evt.waku_message;
                    let payload = message.payload.to_vec().clone();
                    match from_utf8(&payload) {
                        Ok(msg) => {
                            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                            println!("Message Received: {}", msg);
                            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");

                            // let tx_inner = tx_cloned.clone();
                            let msg_inner = msg.to_string();
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

    waku.relay_subscribe(&topic).expect("waku should subscribe");

    // let target_node_multi_addr =
    //   "/dns4/store-01.do-ams3.status.staging.status.im/tcp/30303/p2p/16Uiu2HAm3xVDaz6SRJ6kErwC21zBJEZjavVXg7VSkoWzaV1aMA3F"
    //   .parse::<Multiaddr>().expect("parse multiaddress");
    let target_node_multi_addr =
      "/ip4/24.144.78.119/tcp/30303/p2p/16Uiu2HAm3xVDaz6SRJ6kErwC21zBJEZjavVXg7VSkoWzaV1aMA3F"
      .parse::<Multiaddr>().expect("parse multiaddress");

    waku.connect(&target_node_multi_addr, None)
        .expect("waku should connect to other node");

    // Welcome the player
    greeting();
    // Draw the field
    draw(&board);

    ask_role_to_user(&mut my_role);
    ask_game_name_to_user(&mut game_name);

    println!("AAAA 1");
    let board_string: String = board.iter().collect();
    println!("AAAA 2");
    let _ = waku.relay_publish_txt(&topic,
        &board_string,
        "tic-tac-toe-example",
        None);
    println!("AAAA 3");

    // Main receiver task
    // tokio::spawn(async move {

    //     loop {
    //         // Draw the field
    //         draw(&board);

    //         if my_role == current_player {
    //             // is my turn
    //             // Ask for user input
    //             ask_user(&mut board, current_player);
    //         }
    //         else {
    //             // other player's turn
    //             println!("Waiting oponent's movement");
    //             while let Some(message) = rx.recv().await {
    //                 println!("Received: {}", message);
    //                 break;
    //             }
    //         }

    //         if game_logic(&mut current_player, &mut board, topic, &waku) {
    //             break;
    //         }

    //         // Switch current_player
    //         current_player = if current_player == 'X' { 'O' } else { 'X' };
    //     }
    // });
    // .await
    // .expect("Receiver task panicked");
}

