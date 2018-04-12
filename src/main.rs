extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate socket2;
extern crate clap;

use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};
use futures::sync::mpsc::{UnboundedSender, unbounded};
use tokio_io::codec::LinesCodec;
use tokio::net::{UdpSocket, UdpFramed};
use tokio::prelude::*;
use clap::{App, Arg};

const DEFAULT_USERNAME: &'static str = "Anonymous";
const DEFAULT_PORT: &'static str = "50692";
const DEFAULT_MULTICAST: &'static str = "239.255.42.98";
const IP_ALL: [u8; 4] = [0, 0, 0, 0];

fn bind_multicast(addr: &SocketAddrV4, multi: &SocketAddrV4)
    -> Result<std::net::UdpSocket, std::io::Error>
{
    use socket2::{Domain, Type, Protocol, Socket};

    let socket = Socket::new(Domain::ipv4(),
        Type::dgram(), Some(Protocol::udp()))?;

    socket.set_reuse_address(true)?;
    socket.bind(&socket2::SockAddr::from(*addr))?;
    socket.set_multicast_loop_v4(true)?;
    socket.join_multicast_v4(
        multi.ip(),
        addr.ip(),
    )?;

    Ok(socket.into_udp_socket())
}

fn read_input(tx: UnboundedSender<String>, username: String) {
    let prefix = format!("{}: ", username);
    let stdin = std::io::stdin();
    loop {
        let mut s = prefix.clone();
        stdin.read_line(&mut s).unwrap();
        tx.unbounded_send(s).unwrap();
    }
}

fn main() {
    let matches = App::new("Udp Multicast Chat")
        .version("0.1.0")
        .author("Henning Ottesen <henning@live.no>")
        .about("Example UDP multicast CLI chat client using Tokio")
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .takes_value(true)
            .default_value(DEFAULT_PORT)
            .help("Sets UDP port number"))
        .arg(Arg::with_name("ip")
            .short("i")
            .long("ip")
            .value_name("IP")
            .takes_value(true)
            .default_value(DEFAULT_MULTICAST)
            .help("Sets multicast IP"))
        .arg(Arg::with_name("username")
            .short("u")
            .long("username")
            .value_name("NAME")
            .takes_value(true)
            .default_value(DEFAULT_USERNAME)
            .help("Sets username"))
        .get_matches();

    let username = matches.value_of("username")
        .unwrap()
        .to_owned();

    let port = matches.value_of("port")
        .unwrap()
        .parse::<u16>()
        .expect("Invalid port number");

    let addr = SocketAddrV4::new(IP_ALL.into(), port);

    let maddr = SocketAddrV4::new(
        matches.value_of("ip")
            .unwrap()
            .parse::<Ipv4Addr>()
            .expect("Invalid IP"),
        port
    );

    assert!(maddr.ip().is_multicast(), "Must be multcast address");

    println!("Starting server on: {}", addr);
    println!("Multicast address: {}\n", maddr);

    let std_socket = bind_multicast(&addr, &maddr)
        .expect("Failed to bind multicast socket");

    let socket = UdpSocket::from_std(std_socket,
        &tokio::reactor::Handle::current()
    ).unwrap();

    let framed = UdpFramed::new(socket, LinesCodec::new());
    let (udp_tx, udp_rx) = Stream::split(framed);
    let (chn_tx, chn_rx) = unbounded::<String>();

    let input_thread =
        std::thread::spawn(move || read_input(chn_tx, username));

    let send = chn_rx
        .map(move |s| (s, SocketAddr::from(maddr)))
        .forward(udp_tx
            .sink_map_err(|e| println!("Error receiving UDP packet: {:?}", e)
        )
    ).map(|_|());

    let recv = udp_rx.for_each(move |(s, _)| {
        println!("{}", s);
        Ok(())
    }).map_err(|e| println!("Error sending UDP packet: {:?}", e));

    let serve = send.select(recv)
        .map(|_|())
        .map_err(|_| ());

    tokio::run(serve);
    input_thread.join().unwrap();
}
