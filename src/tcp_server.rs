//! Простейший многопоточный TCP-сервер с общей историей сообщений.
//!
//! Поведение:
//! - Слушает `127.0.0.1:12345`;
//! - На каждое подключение создаёт поток;
//! - Читает байты порциями по 1024, превращает в UTF-8 строку,
//!   складывает в общую `Vec<String>` под `Arc<Mutex<...>>`,
//!   и отправляет клиенту всю историю (`join("\n") + "\n"`).
//!
//! Запуск:
//! ```bash
//! cargo run --release --bin tcp_server
//! ```
//!
//! Замечания по надёжности:
//! - Нет фрейминга сообщений (чтение может вернуть половину/две склейки);
//! - Один `Mutex` может стать узким местом под высокой нагрузкой;
//! - Нет таймаутов, нет ограничений на длину истории.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// Обрабатывает одно клиентское подключение.
///
/// Читает данные из `stream` в буфер 1024 байта, добавляет сообщение в общую историю
/// и отправляет клиенту всю историю целиком.
///
/// Завершает работу при `read == 0` (клиент закрыл соединение) или при ошибке.
fn handle_client(mut stream: TcpStream, messages: Arc<Mutex<Vec<String>>>) {
    let peer = stream.peer_addr().unwrap();
    println!("Пользователь с адресом: {} подключился к серверу", peer);

    // NOTE: 1024 — размер разового чтения; одного read() может быть недостаточно.
    let mut buffer: [u8; 1024] = [0; 1024];

    loop {
        match stream.read(&mut buffer) {
            Ok(0) => {
                println!("Пользователь {} отключился", peer);
                break;
            }
            Ok(size) => {
                // Преобразование в строку «как есть». Нормализация/валидность не проверяются.
                let message = String::from_utf8_lossy(&buffer[..size]).trim().to_string();
                println!("{} → {}", peer, message);

                {
                    let mut msgs = messages.lock().unwrap();
                    msgs.push(message);
                }

                // Собираем ответ: всю историю сообщений, разделённую \n.
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

/// Точка входа сервера: bind → accept-цикл → spawn обработчиков.
///
/// Ошибки бинда приводят к panic через `expect`. Ошибки accept логируются.
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
