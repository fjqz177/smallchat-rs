// 导入tokio相关的异步I/O特性，包括异步读写流和缓冲读取
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

// `tokio::main` 宏，表示异步main函数，入口点为异步程序
#[tokio::main]
async fn main() {
    // 绑定TCP监听器到本地地址和端口7711
    let listener = TcpListener::bind("0.0.0.0:7711").await.unwrap();

    // 创建一个广播通道，可以向订阅者广播消息
    let (tx, _rx) = broadcast::channel(10);

    // 用来共享和更新客户端信息的线程安全的HashMap
    let clients = Arc::new(RwLock::new(HashMap::<usize, String>::new()));

    // 用来给客户端分配唯一ID的计数器，此方法在实际生产中不推荐
    let mut next_id = 0;

    println!("Server running on port 7711");

    loop {
        // 接收新的TCP连接
        let (mut socket, addr) = listener.accept().await.unwrap();

        println!("New connection: {}", addr);

        // 克隆一份发送端，供新的客户端使用
        let tx = tx.clone();
        // 为新的客户端订阅接收通道
        let mut rx = tx.subscribe();

        // 克隆clients的Arc以在异步块中使用
        let clients = Arc::clone(&clients);

        // 为每个新的连接创建唯一的客户端ID
        let client_id = {
            let mut clients = clients.write().unwrap();
            next_id += 1;
            clients.insert(next_id, format!("user:{}", next_id));
            next_id
        };

        // 对每个连接单独启动一个异步任务
        tokio::spawn(async move {
            // 分隔TCP流为读取器和写入器
            let (reader, mut writer) = socket.split();

            // 对读取器部分进行缓冲
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // 向新客户端发送欢迎消息
            let welcome_message = format!(
                "Welcome! Your ID is {}. Use '/nick NAME' to set a nickname.\n",
                client_id
            );
            writer.write_all(welcome_message.as_bytes()).await.unwrap();

            // 启动一个事件循环，直到客户端断开连接
            loop {
                tokio::select! {
                    // 读取客户端信息
                    read_result = reader.read_line(&mut line) => {
                        match read_result {
                            Ok(0) => break, // 客户端断开连接
                            Ok(_) => {
                                // 处理客户端发来的信息
                                handle_client_message(client_id, &line, &clients, tx.clone()).await;
                                // 重置line，为下一条信息做准备
                                line.clear();
                            },
                            Err(e) => {
                                // 出现读取错误
                                eprintln!("An error occurred while reading from socket: {:?}", e);
                                break;
                            },
                        }
                    },

                    // 接收其他客户端发来的信息
                    recv_result = rx.recv() => {
                        match recv_result {
                            Ok((sender_id, msg)) => {
                                // 仅转发不是由自己发送的信息
                                if sender_id != client_id {
                                    writer.write_all(msg.as_bytes()).await.expect("Write to socket failed.");
                                    writer.flush().await.expect("Flush to socket failed.");
                                }
                            },
                            Err(e) => {
                                // 接收错误
                                eprintln!("An error occurred while receiving: {:?}", e);
                                break;
                            },
                        }
                    }
                }
            }

            // 打印断开连接的客户端信息
            println!("Client {} disconnected", client_id);
            {
                // 从HashMap中移除断开连接的客户端
                let mut clients = clients.write().unwrap();
                clients.remove(&client_id);
            }
        });
    }
}

// 用于处理发送者的消息，可能会更新昵称，或者广播消息给其他客户端
async fn handle_client_message(
    client_id: usize,
    line: &str,
    clients: &Arc<RwLock<HashMap<usize, String>>>,
    tx: broadcast::Sender<(usize, String)>,
) {
    if line.starts_with("/nick ") {
        // 用户尝试设置新的昵称
        let nick = line[6..].trim(); // 提取新昵称，假设命令格式总是正确的
        let mut clients = clients.write().unwrap(); // 获取写锁
        clients.insert(client_id, nick.to_string()); // 更新HashMap中的昵称
    } else {
        // 其他信息将被广播给所有客户端
        let clients = clients.read().unwrap(); // 获取读锁
        let nick = clients.get(&client_id).unwrap(); // 获取发送者的昵称
        let msg = format!("{}> {}", nick, line); // 格式化消息
        tx.send((client_id, msg)).unwrap(); // 发送消息
    }
}
