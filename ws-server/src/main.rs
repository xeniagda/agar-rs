#[macro_use]
extern crate lazy_static;

extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate tokio;
extern crate futures;
extern crate agar_backend;
extern crate serde_json;


use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::io::{Error, ErrorKind};
use std::thread;

use tokio::net::TcpListener;
use tokio::timer::Interval;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

use futures::{Future, Stream, Sink};
use futures::sync::mpsc::unbounded;

use agar_backend::{State, Player};

lazy_static! {
    static ref STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::new()));
    static ref PLAYER_ADDR_ID: Mutex<Vec<(SocketAddr, usize)>> = Mutex::new(Vec::new());
}

fn main() {
    let addr: SocketAddr = ([127, 0, 0, 1], 6969).into();

    let server = TcpListener::bind(&addr).expect("Can't make server");

    thread::spawn(run_state_manager);

    let f = server.incoming()
        .map_err(|e| {
            Error::new(ErrorKind::Other, e)
        })
        .and_then(|stream| {
            let addr = stream.peer_addr().unwrap();
            println!("Connection from {:?}", addr);

            accept_async(stream)
                .map(move |ws| (ws, addr))
                .map_err(|e| Error::new(ErrorKind::Other, e))
        })
        .for_each(|(ws_stream, addr)| {
            println!("Websocket connection from {:?}", addr);

            let (sink, stream) = ws_stream.split();

            let (mut sender, recv) = unbounded();

            let mut pinger_sender = sender.clone();
            let pinger = Interval::new(Instant::now(), Duration::from_secs(1))
                    .for_each(move |_| {
                        if let Ok(state) = STATE.lock() {
                            let json = serde_json::to_string(&*state).expect("Can't jsonise the state!");
                            pinger_sender.start_send(Message::Text(json));
                        }

                        Ok(())
                    })
                    .map_err(|_| ());

            let send = recv.fold(
                sink,
                |mut sink, msg| {
                    sink.start_send(msg).expect("Can't send!");
                    Ok(sink)
                })
                .map(|_| ());

            tokio::spawn(pinger);
            tokio::spawn(send);

            if let Ok(mut state) = STATE.lock() {
                if let Ok(mut player_addr_id) = PLAYER_ADDR_ID.lock() {
                    let highest_id = player_addr_id.iter().map(|(_, id)| *id).max().unwrap_or(0);
                    let new_id = highest_id + 1;
                    player_addr_id.push((addr, new_id));
                    state.players.insert(new_id, Player { pos: (0., 0.), direction: 0., speed: 5., size: 5. });
                    sender.start_send(Message::Text(format!("{}", new_id)));
                }
            }


            Ok(())
        })
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        });

    tokio::run(f);
}

fn run_state_manager() {
    let state_manager = Interval::new(Instant::now(), Duration::from_millis(20))
            .fold(None, |last, now| {
                match last {
                    None => Ok(Some(now)),
                    Some(last) => {
                        let since_last = now.duration_since(last);
                        let dt = since_last.as_secs() as f64 + since_last.subsec_nanos() as f64 * 1e-9;

                        if let Ok(mut state) = STATE.lock() {
                            state.tick(dt);
                        }

                        Ok(Some(now))
                    }
                }
            })
            .map(|_| ())
            .map_err(|_| ());

    tokio::run(state_manager);
}
