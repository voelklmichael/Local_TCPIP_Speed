use std::io::{Read, Write};
//use std::net::{TcpListener, TcpStream};
use mio::net::{TcpListener, TcpStream};

const N: usize = 250;
const NODELAY: bool = true;

fn read(stream: &mut TcpStream, msg: &str) {
    let mut response = [0; 1];
    loop {
        match stream.read(&mut response) {
            Ok(1) => {
                if response[0] == 42 {
                    break;
                } else {
                    panic!("Wrong message @ {:?}", msg)
                }
            }
            Ok(0) => continue,
            Ok(n) => panic!("Wrong message length {:} @ {:}", n, msg),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => panic!(e),
        }
    }
}

fn server_startup(address: &str) -> Vec<std::time::Instant> {
    let mut times = Vec::with_capacity(3 * N + 4);
    let address: std::net::SocketAddr = address.parse().expect("Failed to parse");
    // startup server
    times.push(std::time::Instant::now());
    let mut server;
    {
        let server_ = TcpListener::bind(address).expect("Failed to setup server");
        loop {
            match server_.accept() {
                Ok((s, _)) => {
                    server = s;
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
                Err(e) => panic!(e),
            }
        }
    }

    times.push(std::time::Instant::now());
    server
        .set_nodelay(NODELAY)
        .expect("Failed to set no_delay @ server");
    times.push(std::time::Instant::now());

    // 10x send message
    for _i in 0..N {
        times.push(std::time::Instant::now());
        // read answer
        read(&mut server, "server");
        // write
        times.push(std::time::Instant::now());
        server.write(&[42]).expect("Failed to write @ server");
        times.push(std::time::Instant::now());
    }
    times.push(std::time::Instant::now());

    times
}

fn client_startup(address: &str) -> Vec<std::time::Instant> {
    let mut times = Vec::with_capacity(3 * N + 4);
    let address: std::net::SocketAddr = address.parse().expect("Failed to parse");
    // startup server
    times.push(std::time::Instant::now());
    let mut client = TcpStream::connect(address).expect("Failed to setup client");
    times.push(std::time::Instant::now());
    client
        .set_nodelay(NODELAY)
        .expect("Failed to set no_delay @ client");
    times.push(std::time::Instant::now());

    // 10x send message
    for _i in 0..N {
        // write
        times.push(std::time::Instant::now());
        client.write(&[42]).expect("Failed to write @ client");
        times.push(std::time::Instant::now());
        // read answer
        read(&mut client, "client");
        times.push(std::time::Instant::now());
    }
    times.push(std::time::Instant::now());

    times
}

fn main() {
    let address = "127.0.0.1:42042";

    let server_thread = std::thread::spawn(move || server_startup(address));
    std::thread::sleep(std::time::Duration::from_millis(250));

    let client_thread = std::thread::spawn(move || client_startup(address));

    let server_times = server_thread
        .join()
        .expect("Failed to join thread for sverer");
    let client_times = client_thread
        .join()
        .expect("Failed to join thread for client");
    println!("------ Evaluation -----");
    assert_eq!(client_times.len(), server_times.len());

    for i in 0..N {
        let cs = &client_times[3 + i * 3..3 + i * 3 + 3];
        let ss = &server_times[3 + i * 3..3 + i * 3 + 3];
        let client_write = (cs[1] - cs[0]).as_nanos();
        let client_read = (cs[2] - cs[1]).as_nanos();
        let client_total = (cs[2] - cs[0]).as_nanos();
        let server_write = (ss[1] - ss[0]).as_nanos();
        let server_read = (ss[2] - ss[1]).as_nanos();
        let server_total = (ss[2] - ss[0]).as_nanos();
        let cs = [client_write, client_read, client_total];
        let ss = [server_write, server_read, server_total];
        println!("{:?}, {:?}", cs, ss);
        /*
        let client_start = client_times[3]
        let start_time = client_times[3 + i * 3];
        let s = server_times[3 + i * 3..3 + i * 3 + 3]
            .into_iter()
            .map(|x| (*x - start_time).as_nanos());
        let c = client_times[3 + i * 3..3 + i * 3 + 3]
            .into_iter()
            .map(|x| (*x - start_time).as_nanos());
        let x = s.zip(c).collect::<Vec<_>>();
        dbg!(x);
        */
    }
}
