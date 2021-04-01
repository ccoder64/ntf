[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`ntf` 可以在运行程序以后进行通知，和 [ntfy](https://github.com/dschep/ntfy)很像，功能更简化

# 使用方式
## 安装
### 源码安装
克隆代码，使用 rust 进行编译
```shell
cargo install --path . --root /usr/local/
```
### 配置
参照 `config/default.toml`,配置使用 toml 格式，配置里主要包括通知消息，目前支持三种通知方式
1. 自己构造 http 请求，可以实现 slack/telegram/pushover 等等通知
2. 企业微信机器人通知
3. 运行 shell 脚本

type 分别对应 `http`,`work_weixin`,`shell`

配置文件位于 /etc/ntf.toml 或者 ~/.ntf.toml
### 运行
```shell
# 测试配置
ntf test 
# 发送测试消息
ntf send
# 运行命令并通知
ntf done ps aux
# 简化命令
alias ntfy='ntf -v done'
ntfy ps aux
```
更多命令参考
```shell
ntf 0.1.0
ccoder64. <ccoder64@gmail.com>
Run program and notify

USAGE:
    ntf [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -v               Sets the level of verbosity
    -V, --version    Prints version information

OPTIONS:
    -b, --backend <backend>    Notify backend service
    -c, --config <config>      Sets a custom config file
    -m, --message <message>    Message body sent
    -t, --title <title>        Message title sent

SUBCOMMANDS:
    done    Execute the command and notify the message
    help    Prints this message or the help of the given subcommand(s)
    send    Test Send Message
    test    Test Configuration
```
