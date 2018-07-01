#[macro_use]
extern crate lazy_static;

extern crate tokio_tungstenite;
extern crate tungstenite;
extern crate tokio;
extern crate futures;
extern crate agar_backend;

#[cfg(feature = "serde_cbor")]
extern crate serde_cbor as serde_impl;

#[cfg(feature = "serde_json")]
extern crate serde_json as serde_impl;



use std::net::{SocketAddr, IpAddr};
use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};
use std::io::{Error, ErrorKind};
use std::env::args;
use std::thread;

use tokio::net::TcpListener;
use tokio::timer::Interval;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

use futures::{Future, Stream, Sink};
use futures::sync::mpsc::unbounded;

use agar_backend::{State, IdPlayerCommand};

lazy_static! {
    static ref STATE: Arc<Mutex<State>> = Arc::new(Mutex::new(State::new()));
    static ref PLAYER_ADDR_ID: Mutex<Vec<(SocketAddr, usize)>> = Mutex::new(Vec::new());
}

fn main() {
    let mut args = args();

    let mut addr: SocketAddr = ([127, 0, 0, 1], 6969).into();
    if let Some(arg) = args.nth(1) {
        if let Ok(x) = arg.parse::<SocketAddr>() {
            addr = x;
        }
        if let Ok(x) = arg.parse::<IpAddr>() {
            addr = SocketAddr::new(x, 6969);
        }
    }

    eprintln!("Starting WebSocket server on {}", addr);

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

            let mut id = 0;
            if let Ok(mut state) = STATE.lock() {
                if let Ok(mut player_addr_id) = PLAYER_ADDR_ID.lock() {
                    let highest_id = player_addr_id.iter().map(|(_, id)| *id).max().unwrap_or(0);
                    id = highest_id + 1;
                    player_addr_id.push((addr, id));
                    state.add_player(id);

                    println!("Added player {:?}", id);
                }
            }

            let pinger = Interval::new(Instant::now(), Duration::from_millis(100))
                    .for_each(move |_| {
                        if let Ok(state) = STATE.lock() {
                            let json = serde_impl::to_vec(&(&*state, id)).expect("Can't jsonise the state!");
                            sender.start_send(Message::Binary(json));
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

            let stream = stream
                    .for_each(move |msg| {
                        if let Message::Binary(json) = msg {
                            if let Ok(cmd) = serde_impl::from_slice::<IdPlayerCommand>(&json) {
                                if cmd.id == id {
                                    if let Ok(mut state) = STATE.lock() {
                                        state.do_command(cmd);
                                    }
                                }
                            }
                        }
                        Ok(())
                    })
                    .map_err(|_| ());

            tokio::spawn(send);
            tokio::spawn(stream);
            tokio::spawn(pinger);



            Ok(())
        })
        .map_err(|e| {
            eprintln!("Error: {:?}", e);
        });

    tokio::run(f);
}

fn run_state_manager() {
    let state_manager = Interval::new(Instant::now(), Duration::from_millis(75))
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
