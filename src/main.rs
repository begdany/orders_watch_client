use serde::Serialize; // Используется для сериализации данных в JSON-объект
use std::io::{Read, Write}; // Используются для чтения/записи даных сетевого соединения
use std::net::TcpStream; // Используется для TCP-соединения с сервером
use tokio_postgres::{NoTls, Error}; // Используется для работы с базой данных
use std::env; // Модуль env применяется для настройки отображения сообщений логирования
use uuid::Uuid;
use std::io; // Для ввода пользователя

// Определяем структуру, содержащую данные о товаре
#[derive(Serialize)]
struct ItemData {
    brand: String,
    name: String,
    price: i64,
    id: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Включаем отображение лога и инициализируем env_logger
    init_logging();
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
    // Объявляем строку в которую будем записывать запрос к серверу
    let mut request: String; 
    // Зацикливаем работу программы
    loop {
        print_commands(); // Выводим комманды управления работой программы
        let input = read_input(); // Считываем введенную пользователем комманду
        // Формируем HTTP-запрос в зависимости от введенной команды
        match input.trim() {
            "r" => {
                // Выполняем запрос к базе данных
                let row = client
                    .query_one("SELECT brand, name, price FROM data WHERE id = $1", &[&1])
                    .await?;
                // Создаем экземпляр структуры
                let data = ItemData {
                    brand: row.get("brand"),
                    name: row.get("name"),
                    price: row.get("price"),
                    id: Uuid::new_v4().to_string(),
                };
                // Сериализуем объект data в JSON
                let json = serde_json::to_string(&data).unwrap();
                // Формируем HTTP-запрос
                request = create_request("POST", "", &json);
            }
            "p" => request = create_request("GET", "/previous", ""),
            "n" => request = create_request("GET", "/next", ""),
            "f" => request = create_request("GET", "/first", ""),
            "l" => request = create_request("GET", "/last", ""),
            _ => break,
        }
        // Создаем TCP-соединение с сервером
        let mut stream = TcpStream::connect("127.0.0.1:3000").unwrap();
        // Отправляем запрос
        stream.write_all(request.as_bytes()).unwrap();
        log::info!("Отправлен запрос на сервер.");
        // Читаем и логируем ответ
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        log::info!("Получен ответ от сервера:\n{}", response);       
    }
  
    Ok(())
}

// Функция включения логирования
fn init_logging() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
}

// Функция вывода комманд управления работой программы
fn print_commands() {
    println!("Введите 'r' для отправки новых данных");
    println!("        'p' для перехода к предыдущим данным");
    println!("        'n' для перехода к следующим данным");
    println!("        'f' для перехода к первым данным");
    println!("        'l' для перехода к последним данным");
    println!("Введите любые другие символы для выхода:");
}

// Функция чтения введенной пользователем комманды
fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input
}

// Функция формирования HTTP запроса
fn create_request(method: &str, route: &str, json: &str) -> String{
    match method {
        "POST" => format!(
            "POST /post HTTP/1.1\r\n\
            Host: 127.0.0.1:3000\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            Connection: close\r\n\r\n\
            {}",
            json.len(),
            json
        ),
        "GET" => format!(
            "GET {} HTTP/1.1\r\n\
            Host: 127.0.0.1:3000\r\n\
            Connection: close\r\n\r\n\
            ",
            route
        ),
        _ => panic!("Неподдерживаемый HTTP метод: {}", method),
    }
}