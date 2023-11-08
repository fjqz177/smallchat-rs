use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener};
use tokio::sync::broadcast;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    // 创建一个 TCP 监听器，默认端口为 7711
    let listener = TcpListener::bind("127.0.0.1:7711").await.unwrap();

    // 创建一个 broadcast channel，来向所有客户端发送消息
    // 我们使用 (usize, String) 元组，usize 表示排除特定的发送者的 ID，String 是消息内容
    let (tx, _rx) = broadcast::channel(10);

    // 使用 Arc 和 RwLock 来共享和更新客户端信息
    // 这里存储每个客户端的昵称，并与其未来（futures）关联
    let clients = Arc::new(RwLock::new(HashMap::<usize, String>::new()));

    // 初始化客户端 ID。实际上，这种递增 ID 分配方式在生产应用中不是很好的方式，因为它很容易预测。
    // 在实际应用中，你会想要使用更安全的 ID 生成机制。
    let mut next_id = 0;

    println!("Server running on port 7711");

    loop {
        // 接受新的连接
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("New connection: {}", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        let clients = Arc::clone(&clients);

        // 分配新的客户端 ID
        let client_id = {
            let mut clients = clients.write().unwrap();
            next_id += 1;
            clients.insert(next_id, format!("user:{}", next_id));
            next_id
        };

        tokio::spawn(async move {
            // 分读写两部分操作
            let (reader, mut writer) = socket.split();

            // 创建带缓冲的读取器
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // 发送欢迎消息
            let welcome_message = format!("Welcome! Your ID is {}. Use '/nick NAME' to set a nickname.\n", client_id);
            writer.write_all(welcome_message.as_bytes()).await.unwrap();

            loop {
                tokio::select! {
                    // 读取客户端发来的消息
                    read_result = reader.read_line(&mut line) => {
                        match read_result {
                            Ok(0) => break, // 客户端断开连接
                            Ok(_) => {
                                // 处理客户端发来的消息
                                handle_client_message(client_id, &line, &clients, tx.clone()).await;
                                // 清空收到的消息以便下一次循环读取
                                line.clear();
                            },
                            Err(e) => {
                                eprintln!("An error occurred while reading from socket: {:?}", e);
                                break;
                            },
                        }
                    },

                    // 接受来自其他客户端的消息，并发送
                    recv_result = rx.recv() => {
                        match recv_result {
                            Ok((sender_id, msg)) => {
                                if sender_id != client_id {
                                    // 客户端不是消息的原始发送者，将其转发出去
                                    writer.write_all(msg.as_bytes()).await.expect("Write to socket failed.");
                                    writer.flush().await.expect("Flush to socket failed.");
                                }
                            },
                            Err(e) => {
                                eprintln!("An error occurred while receiving: {:?}", e);
                                break;
                            },
                        }
                    }
                }
            }

            println!("Client {} disconnected", client_id);
            {
                // 客户端断开连接，移除其在 HashMap 中的条目
                let mut clients = clients.write().unwrap();
                clients.remove(&client_id);
            }
        });
    }
}

// 处理客户端消息，并在必要时更新昵称
async fn handle_client_message(
    client_id: usize,
    line: &str,
    clients: &Arc<RwLock<HashMap<usize, String>>>,
    tx: broadcast::Sender<(usize, String)>,
) {
    if line.starts_with("/nick ") {
        // 设置或更新客户端的昵称
        let nick = line[6..].trim(); // 假定命令格式总是正确的
        let mut clients = clients.write().unwrap();
        clients.insert(client_id, nick.to_string());
    } else {
        // 否则就转发消息到其他客户端
        let clients = clients.read().unwrap();
        let nick = clients.get(&client_id).unwrap();
        let msg = format!("{}> {}", nick, line);
        tx.send((client_id, msg)).unwrap();
    }
}