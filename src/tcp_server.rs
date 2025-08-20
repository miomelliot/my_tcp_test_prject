use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

fn handle_client(mut stream: TcpStream, messages: Arc<Mutex<Vec<String>>>) {
    let peer = stream.peer_addr().unwrap();
    println!("Пользователь с адресом: {} подключился к серверу", peer);

    let mut buffer: [u8; 1024] = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Пользователь {} отключился", peer);
                break;
            }
            Ok(size) => {
                let message = String::from_utf8_lossy(&buffer[..size]).trim().to_string();
                println!("{} → {}", peer, message);

                {
                    let mut msgs = messages.lock().unwrap();
                    msgs.push(message);
                }

                let response = {
                    let msgs = messages.lock().unwrap();
                    msgs.join("\n") + "\n"
                };

                if let Err(e) = stream.write_all(response.as_bytes()) {
                    eprintln!("Ошибка отправки ответа клиенту {}: {}", peer, e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Ошибка при чтении от клиента {}: {}", peer, e);
                break;
            }
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").expect("Не удалось запустить сервер");
    println!("Сервер запущен на 127.0.0.1:12345");

    let messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let msgs = Arc::clone(&messages);
                thread::spawn(move || {
                    handle_client(stream, msgs);
                });
            }
            Err(e) => eprintln!("Ошибка при подключении клиента: {}", e),
        }
    }
}
