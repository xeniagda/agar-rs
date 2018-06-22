extern crate hyper;
extern crate tokio;

use std::net::SocketAddr;
use std::io::Read;
use std::path::{Path, PathBuf};

use hyper::{Request, Response, Body};
use hyper::Server;
use hyper::service::service_fn;
use hyper::rt::Future;
use tokio::runtime::Runtime;
use tokio::fs::File;

const FILE_PATH: &str = "../client/site/";

fn main() {
    let mut rt = Runtime::new().expect("Can't make runtime");
    start_http(&mut rt);
    rt.shutdown_on_idle().wait().unwrap();
}

fn start_http(rt: &mut Runtime) {
    let http_addr: SocketAddr = ([127, 0, 0, 1], 8080).into();


    let server =
        Server::bind(&http_addr)
        .serve(|| service_fn(handle_request))
        .map_err(|e| eprintln!("Error: {:?}", e));


    rt.spawn(server);
}

fn handle_request(req: Request<Body>) -> impl Future<Item = Response<Body>, Error=String> {
    println!("URI: {:?}", req.uri().path());

    let mut uri = &req.uri().path()[1..];
    if uri.is_empty() {
        uri = "index.html";
    }

    let mut path = PathBuf::new();
    path.push(FILE_PATH);
    path.push(uri);

    println!("Content-Type: {}", get_content_type(&path));

    File::open(path.clone())
        .map_err(|e| format!("Error: {:?}", e))
        .map(move |mut f| {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).expect("Can't read file");

            Response::builder()
                .header("Content-Type", &*get_content_type(&path))
                .body(buf.into())
                .unwrap()
        })
}

fn get_content_type(path: &Path) -> String {
    match path.extension().and_then(|x| x.to_str()) {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        _ => "text/plain",
    }.into()
}

/*
fn start_ws(rt: &mut Runtime) {
    let ws_addr: SocketAddr = ([127, 0, 0, 1], 6969).into();

    let server = TcpListener::bind(&ws_addr).expect("Can't make server");


    let f = server.incoming()
        .map_err(|e| {
            eprintln!("Incoming error: {:?}", e);
        })
        .for_each(|stream| {
            println!("Connection from {:?}", stream.peer_addr());

            accept_async(stream)
            .map_err(|e| {
                eprintln!("Handshake error: {:?}", e);
            })
            .map(|wstream| {
                let (sink, stream) = wstream.split();

                let (tx, rx) = unbounded::<Message>();

                tx.unbounded_send(Message::Text("Hello World".into())).unwrap();

                let timed_sender = Interval::new(Instant::now(), Duration::new(1, 0))
                        .map_err(|e| Error::new(ErrorKind::Other, e))
                        .for_each(move |_|
                                 tx.unbounded_send(Message::Text("Ping".into()))
                                 .map(|_| ())
                                 .map_err(|e| Error::new(ErrorKind::Other, e))
                                );

                let recv = stream
                        .for_each(|msg| {
                            println!("Received {:?}", msg);
                            Ok(())
                        })
                        .map_err(|e| Error::new(ErrorKind::Other, e));

                let sender =
                        rx.fold(sink, |mut sink, msg| {
                            sink.start_send(msg).unwrap();
                            Ok(sink)
                        })
                        .map(|_| ())
                        .map_err(|_| Error::new(ErrorKind::Other, "rx error"));

                let all =
                        timed_sender.join(recv).join(sender)
                        .map(|_| ())
                        .map_err(|e| panic!("Error: {:?}", e));

                tokio::spawn(all);
            })
        });

    rt.spawn(f);
}
*/
