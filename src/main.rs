use serde::Serialize; // Используется для сериализации данных в JSON-объект
use std::io::{Read, Write}; // Используются для чтения/записи даных сетевого соединения
use std::net::TcpStream; // Используется для TCP-соединения с сервером

// Определяем структуру, содержащую данные о товаре
#[derive(Serialize)]
struct ItemData {
    brand: String,
    name: String,
    price: u32,
}

fn main() {
    // Создаем экземпляр структуры
    let data = ItemData {
        brand: "Outleap".to_string(),
        name: "RUDEWAY CRM2".to_string(),
        price: 93307,
    };

    // Сериализуем объект data в JSON
    let json = serde_json::to_string(&data).unwrap();

    // Создаем TCP-соединение с сервером
    let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();

    // Формируем HTTP-запрос
    let request = format!(
        "POST /post HTTP/1.1\r\n\
         Host: 127.0.0.1:3000\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\r\n\
         {}",
        json.len(),
        json
    );

    // Отправляем запрос
    stream.write_all(request.as_bytes()).unwrap();

    // Читаем ответ
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    // Выводим ответ в консоль
    println!("Ответ от сервера:\n{}", response);
}