use serde::Serialize; // Используется для сериализации данных в JSON-объект
use std::io::{Read, Write}; // Используются для чтения/записи даных сетевого соединения
use std::net::TcpStream; // Используется для TCP-соединения с сервером
use tokio_postgres::{NoTls, Error}; // Используется для работы с базой данных
use std::env; // Модуль env применяется для настройки отображения сообщений логирования
use rand::Rng; // Для генерации id
use std::io; // Для ввода пользователя

// Определяем структуру, содержащую данные о товаре
#[derive(Serialize)]
struct ItemData {
    brand: String,
    name: String,
    price: i64,
    id: u8,
}

#[tokio::main]
async fn main() -> Result<(), Error> {

    env::set_var("RUST_LOG", "info"); // Включаем отображение сообщений лога
    env_logger::init(); // Инициализируем систему логирования env_logger

    // Подключаемся к базе данных
    let (client, connection) =
        tokio_postgres::connect("host=localhost user=postgres password=1", NoTls).await?;
    log::info!("Подключение к базе данных...");

    // Запускаем соединение в отдельном асинхронном потоке
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("Ошибка подключения: {}", e);
        } else {
            log::info!("Подключение прошло успешно.");
        }
    });

    loop {

        // Создаем TCP-соединение с сервером
        let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();
        let request: String;

        // Вывод сообщения управления работы программой
        println!("Введите 'r' для отправки новых данных");
        println!("        'p' для перехода к предыдущим данным");
        println!("        'n' для перехода к следующим данным");
        println!("        'l' для перехода к последним данным");
        println!("Введите любые другие символы для выхода:");

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.trim() {
            "r" => { // Повторяем выполнение программы

                // Выполняем запрос к базе данных
                let row = client
                    .query_one("SELECT brand, name, price FROM data WHERE id = $1", &[&1])
                    .await?;

                // Создаем экземпляр структуры
                let data = ItemData {
                    brand: row.get("brand"),
                    name: row.get("name"),
                    price: row.get("price"),
                    id: rand::thread_rng().gen_range(0..255),
                };

                // Сериализуем объект data в JSON
                let json = serde_json::to_string(&data).unwrap();
            
                // Формируем HTTP-запрос
                request = format!(
                    "POST /post HTTP/1.1\r\n\
                    Host: 127.0.0.1:3000\r\n\
                    Content-Type: application/json\r\n\
                    Content-Length: {}\r\n\
                    Connection: close\r\n\r\n\
                    {}",
                    json.len(),
                    json
                );

            }
            "p" => {
                request = format!(
                    "GET /previous HTTP/1.1\r\n\
                    Host: 127.0.0.1:3000\r\n\
                    Connection: close\r\n\r\n\
                    "
                );               
            }
            "n" => {
                request = format!(
                    "GET /next HTTP/1.1\r\n\
                    Host: 127.0.0.1:3000\r\n\
                    Connection: close\r\n\r\n\
                    "
                );               
            }
            "f" => {
                request = format!(
                    "GET /first HTTP/1.1\r\n\
                    Host: 127.0.0.1:3000\r\n\
                    Connection: close\r\n\r\n\
                    "
                );               
            }
            "l" => {
                request = format!(
                    "GET /last HTTP/1.1\r\n\
                    Host: 127.0.0.1:3000\r\n\
                    Connection: close\r\n\r\n\
                    "
                );               
            }
            _ => {
                break;
            }
        }

        // Отправляем запрос
        stream.write_all(request.as_bytes()).unwrap();
        log::info!("Отправлен запрос на сервер.");

        // Читаем ответ
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        // Выводим ответ в консоль
        log::info!("Получен ответ от сервера:\n{}", response);       
    }
  
    Ok(())
}