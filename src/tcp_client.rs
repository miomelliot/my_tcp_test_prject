//! Простая нагрузочная утилита-клиент для TCP-сервера из этого проекта.
//!
//! Поведение:
//! - Создаёт 15 потоков;
//! - Каждый поток подключается к `127.0.0.1:12345`, отправляет одну строку,
//!   читает ответ (историю сообщений) и печатает её в stdout.
//!
//! Протокол: TCP, UTF-8, без фрейминга. Сервер возвращает историю, разделённую `\n`.
//!
//! Запуск:
//! ```bash
//! cargo run --release --bin tcp_client
//! ```

use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// Отправляет одно сообщение на сервер и печатает ответ.
///
/// Подключается к `127.0.0.1:12345`, отправляет `message`,
/// читает до 4096 байт ответа и выводит их.
///
/// # Ошибки
/// Любая ошибка подключения/записи/чтения печатается в stderr,
/// функция просто завершает работу (паники нет).
fn test_client(message: &str) {
    match TcpStream::connect("127.0.0.1:12345") {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(message.as_bytes()) {
                eprintln!("Ошибка отправки: {}", e);
                return;
            }

            // NOTE: Если ответ длиннее 4096 байт, он будет усечён.
            // Для полного чтения — делайте цикл read() до EOF/0.
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

/// Многопоточный мини-стресс-тест: 15 клиентов, задержка 100 мс между стартами.
///
/// Каждый поток отправляет строку вида `Привет сервер-{i}` и печатает историю.
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

/// Точка входа: запускает `multiple_clients_test`.
fn main() {
    multiple_clients_test();
}
