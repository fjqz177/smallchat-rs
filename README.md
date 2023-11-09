# smallchat-rs

## 简介
### smallchat-rs是[smallchat](https://github.com/antirez/smallchat)的Rust实现

本项目使用`tokio`异步编程库，并尽量还原了[smallchat](https://github.com/antirez/smallchat)的功能，并加上了一些中文注释。
本项目是我用来学习Rust异步编程和网络编程的一个小试水，我只是一个Rust新手，欢迎诸位大佬多多指点。

## 使用方法

### 编译

首先
```
git clone https://github.com/fjqz177/smallchat-rs.git
```

然后在项目的根目录打开终端
```
cargo run # 可以加上 --release 选项
```

### 连接程序准备

#### Windows平台

请在`控制面板`->`程序`->`启用或关闭Windows功能`中勾选`Telnet客户端`选项 (无需重启)

在终端中输入命令
```
telnet localhost 7711
```

若要断开连接，请先按`Ctrl`+`]`进入命令界面，然后输入`q`并回车

#### Linux平台

请使用你的**Linux发行版**自带的**包管理器**安装`netcat`

在终端中输入命令
```
nc localhost 7711
```

## 使用提示

- 本程序暂不支持中文及其相关字符，目前只能使用英文及其符号聊天，否则连接会自动断开
- 连接上以后可以会提示你使用`/nick 你的昵称`命令来设置当前连接用户名称
- 若不使用`/nick`命令设置用户名称，则用户名默认为`user:ID`,`ID`在提示里有写，其本质是第**n**个用户连接
- 可以在一台机器上开启多个终端窗口，输入相同的命令连接来测试聊天功能

## 待办事项

- [ ] 让程序支持中文聊天
- [ ] 给程序做一个Web前端页面(~~这个不在我的能力范围内~~)

## 其他

为了完成这个程序，我使用了`GPT-4 Turbo`和`Claude 2`来辅助我，这两个AI的编程能力真的好强，完全不是`GPT-3.5 Turbo`能比的。