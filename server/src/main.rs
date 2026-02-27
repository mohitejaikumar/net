
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const DEFAULT_PORT: u16 = 28333;

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Received request:\n{}", request);

            if let Some(first_line) = request.lines().next() {
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "GET" {
                    let path = parts[1];

                    let server_root = std::env::current_dir().unwrap();
                    let requested_path = path.trim_start_matches('/');
                    if requested_path.is_empty() {
                        let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 13\r\nConnection: close\r\n\r\n404 not found";
                        let _ = stream.write_all(response.as_bytes());
                        return;
                    }

                    let full_path = server_root.join(requested_path);
                    let canonical = std::fs::canonicalize(&full_path);
                    match canonical {
                        Ok(canon_path) => {
                            if !canon_path.starts_with(&server_root) {
                                let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 13\r\nConnection: close\r\n\r\n404 not found";
                                let _ = stream.write_all(response.as_bytes());
                                return;
                            }

                            match std::fs::read(&canon_path) {
                                Ok(data) => {
                                    let filename = canon_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                    let content_type = if filename.ends_with(".html") {
                                        "text/html"
                                    } else if filename.ends_with(".txt") {
                                        "text/plain"
                                    } else {
                                        "application/octet-stream"
                                    };
                                    println!("File '{}' read successfully ({} bytes), Content-Type: {}", filename, data.len(), content_type);

                                    let response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                        content_type,
                                        data.len()
                                    );

                                    if let Err(e) = stream.write_all(response.as_bytes()) {
                                        eprintln!("Failed to send response headers: {}", e);
                                        return;
                                    }

                                    if let Err(e) = stream.write_all(&data) {
                                        eprintln!("Failed to send file data: {}", e);
                                    }
                                }
                                Err(_) => {
                                    let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 13\r\nConnection: close\r\n\r\n404 not found";
                                    let _ = stream.write_all(response.as_bytes());
                                }
                            }
                        }
                        Err(_) => {
                            
                            let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 13\r\nConnection: close\r\n\r\n404 not found";
                            let _ = stream.write_all(response.as_bytes());
                        }
                    }
                    
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to read from connection: {}", e);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let port = if args.len() > 1 {
        args[1].parse::<u16>().unwrap_or(DEFAULT_PORT)
    } else {
        DEFAULT_PORT
    };
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).expect("Could not bind to address");
    println!("Listening on http://{}", addr);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
