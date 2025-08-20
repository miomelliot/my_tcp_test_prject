use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

fn test_client(message: &str) {
    match TcpStream::connect("127.0.0.1:12345") {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Ошибка отправки: {}", e);
                return;
            }

            let mut buffer: [u8; 4096] = [0; 4096];
            match stream.read(&mut buffer) {
                Ok(size) => {
                    let response = String::from_utf8_lossy(&buffer[..size]);
                    println!("Ответ от сервера:\n{}", response);
                }
                Err(e) => eprintln!("Ошибка чтения ответа: {}", e),
            }
        }
        Err(e) => eprintln!("Ошибка подключения: {}", e),
    }
}

fn multiple_clients_test() {
    let mut handles = vec![];

    for i in 0..15 {
        let msg = format!("Привет сервер-{}", i);
        let handle = thread::spawn(move || {
            test_client(&msg);
        });
        handles.push(handle);
        thread::sleep(Duration::from_millis(100));
    }

    for h in handles {
        let _ = h.join();
    }
}

fn main() {
    multiple_clients_test();
}
