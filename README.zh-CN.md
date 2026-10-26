# herdr-dumb

**herdr-dumb** 是 [herdr](https://github.com/herdrdev/herdr) 的精简加固分支，面向 Microsoft 365 GCC High 等受限环境。它保留 herdr 为本机运行的智能体所提供的全部能力，并移除了可访问网络、运行第三方代码或将外部数据写入磁盘的功能。

目标是让 IT 或安全审查人员能够批准它：没有远程控制、没有自我修改、没有扩展代码，除了你自己打开的终端之外，数据没有其他进出途径。

## 移除了什么，以及原因

| 已移除 | 在上游中的作用 | 移除原因 |
| --- | --- | --- |
| **SSH 机器和远程桥接** | 连接已保存的 SSH 机器，在远程主机上运行 herdr 桥接，并在本地窗口中显示远程窗格和智能体 | 会建立出站连接，并让一台机器操控另一台机器上的终端。这是需要单独审查的远程控制和数据外泄通道 |
| **自更新和通道切换**（`herdr update`、`herdr channel`） | 从 GitHub 发布页下载并安装新的 herdr 二进制文件 | 程序不得自行替换。已安装软件只应通过经过审查批准的发布版本变更 |
| **智能体检测清单下载** | 运行时从 herdr.dev 获取更新的智能体检测规则 | 这是安装后改变行为的网络调用。herdr-dumb 只使用编译进二进制文件的清单 |
| **插件**（`herdr plugin`、插件窗格、插件快捷键动作、插件市场） | 在 herdr 内安装并运行第三方插件代码 | 插件是来自已审查源码之外、未经沙箱隔离的代码。插件运行时、清单加载和注册表均已删除；通过冻结线协议到达的插件动作会被拒绝 |
| **剪贴板图像转发** | 将客户端粘贴的图像写入临时目录，并把文件路径输入到窗格中 | 它会把外部数据写入磁盘，并交给窗格中运行的程序。为保持兼容，线协议消息保留，但服务器会忽略它 |

## 保留了什么

分离后继续运行的本地会话、会话恢复、智能体状态检测（working / blocked / idle）、CLI 和本地 socket api、键盘和鼠标控制、网页链接以及文本剪贴板。窗格以你自己的权限运行你启动的程序；herdr-dumb 不对其进行沙箱隔离。

## 与上游的区别

- 可执行文件名为 `herdr-dumb`（Windows 上为 `herdr-dumb.exe`）。
- 配置和状态保存在 `herdr-dumb` 目录中（调试构建为 `herdr-dumb-dev`），因此 herdr-dumb 不会读取或修改已有的 herdr 安装。
- 上游安装脚本、Homebrew、mise 以及 herdr.dev 文档描述的是原版 herdr，而不是此分支。请从本仓库的[发布页](https://github.com/patbonecrusher/herdr-dumb/releases)下载 herdr-dumb，或自行构建（见[开发](#开发)）。
- 内置智能体指南（`src/agent-guide.md`）只记录此分支保留的命令。

本 README 的其余部分来自上游，描述已移除功能的地方已作修改。

---

# herdr


<p align="center">
  <img src="assets/logo.png" alt="herdr" width="100" />
</p>

<p align="center">
  <a href="https://herdr.dev">herdr.dev</a> · <a href="#安装">安装</a> · <a href="https://herdr.dev/zh-cn/docs/quick-start/">快速开始</a> · <a href="https://herdr.dev/zh-cn/docs/">文档</a></p>

<p align="center">
  <a href="README.md">English</a> · 简体中文
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-666666?labelColor=333333" alt="Apache 2.0 license" /></a>
  <a href="https://github.com/herdrdev/herdr/releases"><img src="https://img.shields.io/github/downloads/herdrdev/herdr/total?labelColor=333333&color=666666" alt="total GitHub release downloads" /></a>
  <a href="https://github.com/herdrdev/herdr/stargazers"><img src="https://img.shields.io/github/stars/herdrdev/herdr?labelColor=333333&color=666666&logo=github" alt="GitHub stars" /></a>
  <a href="https://github.com/herdrdev/herdr/releases/latest"><img src="https://img.shields.io/github/v/release/herdrdev/herdr?label=release&labelColor=333333&color=666666" alt="latest stable release" /></a>
  <a href="https://formulae.brew.sh/formula/herdr"><img src="https://img.shields.io/homebrew/v/herdr?label=homebrew&labelColor=333333&color=666666" alt="Homebrew version" /></a>
  <a href="https://x.com/herdrdev"><img src="https://img.shields.io/badge/follow-%40herdrdev-000000?logo=x&logoColor=white" alt="follow @herdrdev on X" /></a>
</p>

---

https://github.com/user-attachments/assets/043ec09f-4bdd-41d5-aee0-8fda6b83e267

**智能体复用器，住在你的终端里。**

- **每个智能体一目了然**——`blocked`、`working`、`done`。真实的终端视图，而不是包装过的转述。
- **分离后工作继续运行**——关闭客户端后，后台服务器仍会保持终端运行。服务器或机器重启后，Herdr 会恢复已保存的布局，并可恢复受支持的智能体会话；原有进程不会保留。[会话状态 →](https://herdr.dev/zh-cn/docs/session-state/)
- **智能体也能使用 herdr**——纯 socket api：智能体可以创建窗格、读取输出、互相等待。[智能体技能 →](https://herdr.dev/zh-cn/docs/agent-skill/)
- **键盘和鼠标都是一等公民**——tmux 风格的前缀键，*以及*点击、拖动、分割。按当下的场景选择，而不是被工具锁死。
- **单个 rust 二进制，没有 electron**——运行在你已经在用的任何终端里。

---

## 安装

从[发布页](https://github.com/patbonecrusher/herdr-dumb/releases)下载适用于你平台的归档：

- `herdr-dumb-windows-x86_64.zip`——保持 `conpty` 目录与 `herdr-dumb.exe` 位于同一位置
- `herdr-dumb-macos-arm64.tar.gz`

然后在工作所在的目录启动它：

```bash
herdr-dumb
```

运行你的智能体、分割窗格，然后安心离开。`ctrl+b q` 分离，`herdr-dumb` 重新连接。[快速开始 →](https://herdr.dev/zh-cn/docs/quick-start/)

## 文档

上游文档位于 [herdr.dev/docs](https://herdr.dev/zh-cn/docs/)，描述的是原版 herdr；请跳过远程机器、更新和插件相关内容：[快速开始](https://herdr.dev/zh-cn/docs/quick-start/) · [核心概念](https://herdr.dev/zh-cn/docs/concepts/) · [受支持的智能体](https://herdr.dev/zh-cn/docs/agents/) · [键盘](https://herdr.dev/zh-cn/docs/keyboard/) · [配置](https://herdr.dev/zh-cn/docs/configuration/) · [会话状态](https://herdr.dev/zh-cn/docs/session-state/) · [会话持久化](https://herdr.dev/zh-cn/docs/persistence-remote/) · [集成](https://herdr.dev/zh-cn/docs/integrations/) · [socket api](https://herdr.dev/zh-cn/docs/socket-api/)

## 致谢

<a href="https://terminaltrove.com/"><img src="assets/sponsors/terminal-trove.png" alt="Terminal Trove" width="200" /></a>

[Terminal Trove](https://terminaltrove.com/) 以及 [SPONSORS.md](./SPONSORS.md) 中列出的每一位支持者——谢谢 🐑

企业/合作：hey@herdr.dev

## 智能体须知

如果你是协助本仓库的 AI 智能体：在改动代码前阅读 [`AGENTS.md`](./AGENTS.md)，在创建 issue 或 PR 前阅读 [`CONTRIBUTING.md`](./CONTRIBUTING.md)。

## 开发

```bash
git clone https://github.com/patbonecrusher/herdr-dumb
cd herdr-dumb
just build       # 生成 ./target/release/herdr-dumb

just test        # 单元测试
just check       # 格式检查、测试和维护性检查
```

## 许可证

herdr 基于 [Apache License 2.0](LICENSE) 许可证发布。
