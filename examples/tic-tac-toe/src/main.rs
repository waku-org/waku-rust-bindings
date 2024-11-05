extern crate termcolor;

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

fn print_player(player: &char) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if player == &'X' {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Blue)))
            .unwrap();
    } else if player == &'O' {
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
            .unwrap();
    }

    write!(&mut stdout, "{}", player).unwrap();
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

fn ask_user(board: &mut [char], player: char) {
    loop {
        print!("Player '");
        print_player(&player);
        println!("', enter a number: ");

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            println!("Couldn't read line! Try again.");
            continue;
        }

        if let Ok(number) = input.trim().parse::<usize>() {
            if number < 1 || number > 9 {
                println!("The field number must be between 1 and 9.");
                continue;
            }

            let number = number - 1;

            if board[number] == 'X' || board[number] == 'O' {
                print!("This field is already taken by '");
                print_player(&board[number]);
                println!("'.");
                continue;
            }

            board[number] = player;

            break;
        } else {
            println!("Only numbers are allowed.");
            continue;
        }
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
fn game_logic(player: &mut char, board: &mut [char],
              topic: &str, waku: &WakuNodeHandle<Running>) -> bool {
    // Ask for user input
    ask_user(board, *player);

    let board_string: String = board.iter().collect();
    let board_str_slice: &str = &board_string;

    let _ = waku.relay_publish_txt(topic, &board_str_slice, "tic-tac-toe-example", None);

    // Check if a player won
    if has_won(&board) {
        draw(&board);
        print!("Player '");
        print_player(&player);
        println!("' won! \\(^.^)/");
        return true;
    }

    // Check if all fields are used
    if is_over(&board) {
        draw(&board);
        println!("All fields are used. No one won. (._.)");
        return true;
    }

    // Switch player
    *player = if *player == 'X' { 'O' } else { 'X' };
    return false;
}

fn main() {
    let mut board = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
    let mut player = 'X';
    let topic = "/waku/2/rs/16/64";

    // Create a Waku instance
    let waku = waku_new(Some(WakuNodeConfig {
        port: Some(60010),
        cluster_id: Some(16),
        ..Default::default()
    }))
    .expect("should instantiate");

    // ctrlc::set_handler(move ||{
    //     println!("Ctrl+C detected. Exiting gracefully...");
    //     // waku.stop();
    // }).expect("Error setting Ctrl+C handler");
    
    let waku = waku.start().expect("waku should start");

    // Establish a closure that handles the incoming messages
    waku.set_event_callback(&|response| {
        if let LibwakuResponse::Success(v) = response {
            let event: Event =
                serde_json::from_str(v.unwrap().as_str()).expect("Parsing event to succeed");

            match event {
                Event::WakuMessage(evt) => {
                    println!("WakuMessage event received: {:?}", evt.waku_message);
                    let message = evt.waku_message;
                    let payload = message.payload.to_vec();
                    match from_utf8(&payload) {
                        Ok(msg) => {
                            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");
                            println!("Message Received: {}", msg);
                            println!("::::::::::::::::::::::::::::::::::::::::::::::::::::");

                            // game_logic(&mut player, &mut board, topic, &waku);
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

    let target_node_multi_addr =
      "/dns4/store-01.do-ams3.status.staging.status.im/tcp/30303/p2p/16Uiu2HAm3xVDaz6SRJ6kErwC21zBJEZjavVXg7VSkoWzaV1aMA3F"
      .parse::<Multiaddr>().expect("parse multiaddress");

    waku.connect(&target_node_multi_addr, None)
        .expect("waku should connect to other node");

    // Welcome the player
    greeting();

    loop {
        // Draw the field
        draw(&board);

        if game_logic(&mut player, &mut board, topic, &waku) {
            break;
        }
    }
}