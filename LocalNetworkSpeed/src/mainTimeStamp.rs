use std::io::{Read, Write};

const N: usize = 15;
fn server(address: &str, start_time: std::time::Instant) {
    // startup server
    let mut server = std::net::TcpListener::bind(address)
        .expect("Failed to setup server")
        .accept()
        .expect("Failed to setup sever @ step 2")
        .0;
    server
        .set_nodelay(true)
        .expect("Failed to set no_delay @ server");

    // 10x send message
    for _ in 0..N {
        // read answer
        let mut response = [0; 16];
        let mut read_bytes = 0;
        loop {
            read_bytes += server
                .read(&mut response[read_bytes..])
                .expect("Failed to read @ server");
            if read_bytes == 16
            /* 16=128/8 */
            {
                break;
            } else if read_bytes > 16 {
                panic!("Too many data read")
            }
        }
        let answer = u128::from_le_bytes(response);
        // write current time
        let now = std::time::Instant::now();
        let elapsed_nanos: u128 = (now - start_time).as_nanos();
        server
            .write(&elapsed_nanos.to_le_bytes())
            .expect("Failed to write @ server");
    }
}

fn main() {
    let address = "127.0.0.1:42042";
    let start_time = std::time::Instant::now();

    let thread = std::thread::spawn(move || server(address, start_time));
    std::thread::sleep(std::time::Duration::from_millis(10));

    // connect client
    let mut client = std::net::TcpStream::connect(address).expect("Failed to setup client");
    client
        .set_nodelay(true)
        .expect("Failed to set no_delay @ client");

    // 10x send message
    let mut times = Vec::with_capacity(N * 2);
    for _ in 0..N {
        // write current time
        let now = std::time::Instant::now();
        let elapsed_nanos: u128 = (now - start_time).as_nanos();
        times.push(elapsed_nanos);
        client
            .write(&elapsed_nanos.to_le_bytes())
            .expect("Failed to write @ client");
        let mut response = [0; 16];
        let mut read_bytes = 0;
        // read answer
        loop {
            read_bytes += client
                .read(&mut response[read_bytes..])
                .expect("Failed to read @ client");
            if read_bytes == 16
            /* 16=128/8 */
            {
                break;
            } else if read_bytes > 16 {
                panic!("Too many data read")
            }
        }
        let answer = u128::from_le_bytes(response);
        times.push(answer);
    }
    // add final time
    {
        let now = std::time::Instant::now();
        let elapsed_nanos: u128 = (now - start_time).as_nanos();
        times.push(elapsed_nanos);
    }

    thread.join().expect("Failed to join threads");
    println!("------ Evaluation -----");
    for i in 0..N {
        let start_time = times[i * 2];
        //let client_write = times[i * 2] - start_time;
        let server_write = times[i * 2 + 1] - start_time;
        let client_next = times[i * 2 + 2] - start_time;
        /*println!(
            "{:02}: Client: {:06} - Server: {:06} - Client: {:06}",
            i, client_write, server_write, client_next
        )*/
        println!(
            "{:02}: Server: {:05} - Client: {:05}",
            i, server_write, client_next
        )
    }
}
