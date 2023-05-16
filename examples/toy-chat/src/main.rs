mod protocol;

use crate::protocol::{Chat2Message, TOY_CHAT_CONTENT_TOPIC};
use chrono::Utc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use prost::Message;
use std::io::Write;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;
use waku_bindings::{
    waku_new, waku_set_event_callback, ContentFilter, Multiaddr, PagingOptions, ProtocolId,
    Running, StoreQuery, WakuMessage, WakuNodeHandle,
};

enum InputMode {
    Normal,
    Editing,
}

const NODES: &[&str] = &[
    "/dns4/node-01.ac-cn-hongkong-c.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAkvWiyFsgRhuJEb9JfjYxEkoHLgnUQmr1N5mKWnYjxYRVm",
    "/dns4/node-01.do-ams3.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmPLe7Mzm8TsYUubgCAW1aJoeFScxrLj8ppHFivPo97bUZ",
    "/dns4/node-01.gc-us-central1-a.wakuv2.test.statusim.net/tcp/30303/p2p/16Uiu2HAmJb2e28qLXxT5kZxVUUoJt72EMzNGXB47Rxx5hw3q4YjS"
];

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    nick: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Arc<RwLock<Vec<Chat2Message>>>,

    node_handle: WakuNodeHandle<Running>,
}

impl App {
    fn new(nick: String, node_handle: WakuNodeHandle<Running>) -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Arc::new(RwLock::new(Vec::new())),
            node_handle,
            nick,
        }
    }
}
fn retrieve_history(
    node_handle: &WakuNodeHandle<Running>,
) -> waku_bindings::Result<Vec<Chat2Message>> {
    let self_id = node_handle.peer_id().unwrap();
    let peer = node_handle
        .peers()?
        .iter()
        .cloned()
        .find(|peer| peer.peer_id() != &self_id)
        .unwrap();

    let result = node_handle.store_query(
        &StoreQuery {
            pubsub_topic: None,
            content_filters: vec![ContentFilter::new(TOY_CHAT_CONTENT_TOPIC.clone())],
            start_time: Some(
                (Duration::from_secs(Utc::now().timestamp() as u64)
                    - Duration::from_secs(60 * 60 * 24))
                .as_nanos() as usize,
            ),
            end_time: None,
            paging_options: Some(PagingOptions {
                page_size: 25,
                cursor: None,
                forward: true,
            }),
        },
        peer.peer_id(),
        Some(Duration::from_secs(10)),
    )?;

    Ok(result
        .messages()
        .iter()
        .map(|waku_message| {
            <Chat2Message as Message>::decode(waku_message.payload())
                .expect("Toy chat messages should be decodeable")
        })
        .collect())
}

fn setup_node_handle() -> std::result::Result<WakuNodeHandle<Running>, Box<dyn Error>> {
    let node_handle = waku_new(None)?;
    let node_handle = node_handle.start()?;
    for address in NODES.iter().map(|a| Multiaddr::from_str(a).unwrap()) {
        let peerid = node_handle.add_peer(&address, ProtocolId::Relay)?;
        node_handle.connect_peer_with_id(&peerid, None)?;
    }
    node_handle.relay_subscribe(None)?;
    Ok(node_handle)
}

fn main() -> std::result::Result<(), Box<dyn Error>> {
    let nick = std::env::args().nth(1).expect("Nick to be set");
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let node_handle = setup_node_handle()?;

    // create app and run it
    let mut app = App::new(nick, node_handle);
    let history = retrieve_history(&app.node_handle)?;
    if !history.is_empty() {
        *app.messages.write().unwrap() = history;
    }
    let shared_messages = Arc::clone(&app.messages);
    waku_set_event_callback(move |signal| match signal.event() {
        waku_bindings::Event::WakuMessage(event) => {
            if event.waku_message().content_topic() != &TOY_CHAT_CONTENT_TOPIC {
                return
            }

            match <Chat2Message as Message>::decode(event.waku_message().payload()) {
                Ok(chat_message) => {
                    shared_messages.write().unwrap().push(chat_message);
                }
                Err(e) => {
                    let mut out = std::io::stderr();
                    write!(out, "{e:?}").unwrap();
                }
            }
        }
        waku_bindings::Event::Unrecognized(data) => {
            let mut out = std::io::stderr();
            write!(out, "Error, received unrecognized event {data}").unwrap();
        }
        _ => {}
    });

    // app.node_handle.relay_publish_message(&WakuMessage::new(Chat2Message::new(&app.nick, format!(""))))
    let res = run_app(&mut terminal, &mut app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    app.node_handle.stop()?;

    if let Err(err) = res {
        println!("{err:?}")
    }
    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> std::result::Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('e') => {
                        app.input_mode = InputMode::Editing;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        let message_content: String = app.input.drain(..).collect();
                        let message = Chat2Message::new(&app.nick, &message_content);
                        let mut buff = Vec::new();
                        let meta = Vec::new();
                        Message::encode(&message, &mut buff)?;
                        let waku_message = WakuMessage::new(
                            buff,
                            TOY_CHAT_CONTENT_TOPIC.clone(),
                            1,
                            Utc::now().timestamp_nanos() as usize,
                            meta,
                            false,
                        );
                        if let Err(e) =
                            app.node_handle
                                .relay_publish_message(&waku_message, None, None)
                        {
                            let mut out = std::io::stderr();
                            write!(out, "{e:?}").unwrap();
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start writing a message."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let messages: Vec<ListItem> = app
        .messages
        .read()
        .unwrap()
        .iter()
        .map(|message| {
            let content = vec![Spans::from(Span::raw(format!(
                "[{} - {}]: {}",
                message.timestamp().unwrap().format("%d-%m-%y %H:%M"),
                message.nick(),
                message.message()
            )))];
            ListItem::new(content)
        })
        .collect();
    let messages = List::new(messages).block(Block::default().borders(Borders::ALL).title("Chat"));
    f.render_widget(messages, chunks[2]);
}
