fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::fs;
    use std::io::{self, Read, Write};
    use std::net::{SocketAddr, TcpListener, TcpStream};
    use std::num::NonZero;
    use std::path::PathBuf;
    use std::thread::{self, JoinHandle};

    fn nz(n: u32) -> NonZero<u32> {
        NonZero::new(n).unwrap()
    }

    fn nz_set(values: &[u32]) -> BTreeSet<NonZero<u32>> {
        values.iter().map(|&v| nz(v)).collect()
    }

    #[test]
    fn test_divisors_of_one() {
        assert_eq!(divisors(nz(1)), nz_set(&[1]));
    }

    #[test]
    fn test_divisors_of_prime() {
        assert_eq!(divisors(nz(13)), nz_set(&[1, 13]));
    }

    #[test]
    fn test_divisors_of_composite() {
        assert_eq!(divisors(nz(12)), nz_set(&[1, 2, 3, 4, 6, 12]));
    }

    #[test]
    fn test_divisors_of_square() {
        assert_eq!(divisors(nz(36)), nz_set(&[1, 2, 3, 4, 6, 9, 12, 18, 36]));
    }

    #[test]
    fn test_assert_sorted_accepts_sorted() {
        assert_sorted(&[1, 2, 3, 10]);
    }

    #[test]
    fn test_assert_sorted_accepts_repeated_values() {
        assert_sorted(&[1, 1, 2, 2]);
    }

    #[test]
    fn test_assert_sorted_accepts_empty_and_single() {
        assert_sorted(&[]);
        assert_sorted(&[7]);
    }

    #[test]
    fn test_assert_sorted_accepts_vec_and_its_part() {
        let mut values: Vec<u32> = (1..=10).collect();
        assert_sorted(&values);
        values[0] = 100;
        assert_sorted(&values[1..]);
    }

    #[test]
    #[should_panic]
    fn test_assert_sorted_panics_on_unsorted() {
        assert_sorted(&[1, 3, 2]);
    }

    #[test]
    #[should_panic]
    fn test_assert_sorted_panics_on_unsorted_end() {
        assert_sorted(&[1, 2, 3, 0]);
    }

    /// Returns both ends of a fresh loopback TCP connection.
    fn connected_pair() -> (TcpStream, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (server, _) = listener.accept().unwrap();
        (client, server)
    }

    #[test]
    fn test_bulk_write_and_bulk_read_large_buffer() {
        // Large enough that a single read or write cannot move all of it at once.
        let data: Vec<u8> = (0..8 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
        let (mut client, mut server) = connected_pair();
        let expected = data.clone();
        let reader = thread::spawn(move || bulk_read(&mut server, expected.len()).unwrap());
        bulk_write(&mut client, &data).unwrap();
        assert_eq!(reader.join().unwrap(), data);
    }

    #[test]
    fn test_bulk_read_exact_size() {
        let (mut client, mut server) = connected_pair();
        client.write_all(b"hello world").unwrap();
        assert_eq!(bulk_read(&mut server, 5).unwrap(), b"hello");
        assert_eq!(bulk_read(&mut server, 6).unwrap(), b" world");
    }

    #[test]
    fn test_bulk_read_zero_bytes() {
        let (_client, mut server) = connected_pair();
        assert_eq!(bulk_read(&mut server, 0).unwrap(), Vec::<u8>::new());
    }

    #[test]
    fn test_bulk_read_fails_on_early_disconnect() {
        let (mut client, mut server) = connected_pair();
        client.write_all(b"abc").unwrap();
        drop(client);
        assert!(bulk_read(&mut server, 10).is_err());
    }

    /// Starts a server for exactly one client, handled by `handle_client`.
    fn serve_one() -> (SocketAddr, JoinHandle<io::Result<()>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_client(stream)
        });
        (addr, handle)
    }

    /// Sends a length-prefixed path and returns the whole reply.
    fn ask(addr: SocketAddr, path: &[u8]) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .write_all(&(path.len() as u32).to_be_bytes())
            .unwrap();
        stream.write_all(path).unwrap();
        let mut reply = String::new();
        stream.read_to_string(&mut reply).unwrap();
        reply
    }

    fn test_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lab4-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir(&dir).unwrap();
        dir
    }

    #[test]
    fn test_handle_client_lists_directory() {
        let dir = test_dir("list");
        fs::write(dir.join("a.txt"), "a").unwrap();
        fs::write(dir.join("b.rs"), "b").unwrap();
        fs::create_dir(dir.join("sub")).unwrap();

        let (addr, handle) = serve_one();
        let reply = ask(addr, dir.to_str().unwrap().as_bytes());
        assert!(handle.join().unwrap().is_ok());
        fs::remove_dir_all(&dir).unwrap();

        let names: BTreeSet<&str> = reply.lines().collect();
        assert_eq!(names, BTreeSet::from(["a.txt", "b.rs", "sub"]));
    }

    #[test]
    fn test_handle_client_lists_empty_directory() {
        let dir = test_dir("empty");
        let (addr, handle) = serve_one();
        let reply = ask(addr, dir.to_str().unwrap().as_bytes());
        assert!(handle.join().unwrap().is_ok());
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(reply, "");
    }

    #[test]
    fn test_handle_client_bad_path() {
        let (addr, handle) = serve_one();
        let reply = ask(addr, &[0xff, 0xfe, 0xfd]);
        assert_eq!(reply, "Bad path\n");
        assert!(handle.join().unwrap().is_ok());
    }

    #[test]
    fn test_handle_client_bad_dir() {
        let missing = std::env::temp_dir().join(format!("lab4-{}-missing", std::process::id()));
        let (addr, handle) = serve_one();
        let reply = ask(addr, missing.to_str().unwrap().as_bytes());
        assert_eq!(reply, "Bad dir\n");
        assert!(handle.join().unwrap().is_ok());
    }

    #[test]
    fn test_handle_client_path_to_file_is_bad_dir() {
        let dir = test_dir("file");
        let file = dir.join("not_a_dir.txt");
        fs::write(&file, "x").unwrap();
        let (addr, handle) = serve_one();
        let reply = ask(addr, file.to_str().unwrap().as_bytes());
        assert!(handle.join().unwrap().is_ok());
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(reply, "Bad dir\n");
    }

    #[test]
    fn test_handle_client_path_sent_in_pieces() {
        let dir = test_dir("pieces");
        fs::write(dir.join("only.txt"), "x").unwrap();
        let path = dir.to_str().unwrap().as_bytes().to_vec();

        let (addr, handle) = serve_one();
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.set_nodelay(true).unwrap();
        let mut message = (path.len() as u32).to_be_bytes().to_vec();
        message.extend_from_slice(&path);
        for piece in message.chunks(3) {
            stream.write_all(piece).unwrap();
            thread::sleep(std::time::Duration::from_millis(5));
        }
        let mut reply = String::new();
        stream.read_to_string(&mut reply).unwrap();
        assert!(handle.join().unwrap().is_ok());
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(reply.lines().collect::<Vec<_>>(), ["only.txt"]);
    }

    #[test]
    fn test_handle_client_returns_error_on_disconnect() {
        let (addr, handle) = serve_one();
        let mut stream = TcpStream::connect(addr).unwrap();
        stream.write_all(&[0, 0]).unwrap();
        drop(stream);
        assert!(handle.join().unwrap().is_err());
    }
}
