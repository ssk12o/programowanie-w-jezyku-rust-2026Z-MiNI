use std::io::{self, Read, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    println!("Input server addres:");
    let mut server_addr = String::new();
    io::stdin()
        .read_line(&mut server_addr)
        .expect("Failed to read a line");

    server_addr = "127.0.0.1:8080".to_string();

    println!("input read path");
    let mut path = String::new();
    io::stdin()
        .read_line(&mut path)
        .expect("Failed to read path");

    path = path.trim().to_string();

    println!("Connecting to {server_addr}");
    let mut stream = TcpStream::connect(server_addr).unwrap();

    println!("Sendint request for path: \"{path}\"");

    let path_bytes = path.as_bytes();
    let len_prefix = (path_bytes.len() as u32).to_be_bytes();

    stream.write_all(&len_prefix)?;
    stream.write_all(path_bytes)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    println!("--- Server said ---");
    print!("{response}");

    Ok(())
}
