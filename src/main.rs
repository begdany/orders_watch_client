use serde::Serialize; // Используется для сериализации данных в JSON-объект
use std::io::{Read, Write}; // Используются для чтения/записи даных сетевого соединения
use std::net::TcpStream; // Используется для TCP-соединения с сервером
use tokio_postgres::{NoTls, Error}; // Используется для работы с базой данных
use std::env; // Модуль env применяется для настройки отображения сообщений логирования

// Определяем структуру, содержащую данные о товаре
#[derive(Serialize)]
struct ItemData {
    brand: String,
    name: String,
    price: i64,
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

    // Выполняем запрос к базе данных
    let row = client
        .query_one("SELECT brand, name, price FROM data WHERE id = $1", &[&1])
        .await?;

    // Создаем экземпляр структуры
    let data = ItemData {
        brand: row.get("brand"),
        name: row.get("name"),
        price: row.get("price"),
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
    log::info!("Отправлен запрос на сервер.");

    // Читаем ответ
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    // Выводим ответ в консоль
    log::info!("Получен ответ от сервера:\n{}", response);

    Ok(())
}