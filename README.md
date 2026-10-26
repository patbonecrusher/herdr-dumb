# herdr-dumb

**herdr-dumb** is a locked-down fork of [herdr](https://github.com/herdrdev/herdr) for restricted environments such as Microsoft 365 GCC High. It keeps everything herdr does for agents running on the local machine, and removes the features that let it reach the network, run third-party code, or write data that came in from outside.

The goal is a tool an IT or security reviewer can approve: no remote control, no self-modification, no extension code, and no path for data to leave or enter except through the terminals you open yourself.

## what was removed, and why

| removed | what it did upstream | why it was removed |
| --- | --- | --- |
| **SSH machines and remote bridges** | connected herdr to saved SSH machines, ran a herdr bridge on the remote host, and showed remote panes and agents in the local window | it opens outbound connections and lets one machine drive terminals on another. that is a remote-control and data-exfiltration channel a reviewer would have to vet separately |
| **self-update and channel switching** (`herdr update`, `herdr channel`) | downloaded and installed new herdr binaries from GitHub releases | the binary must not replace itself. installed software should only change through an approved, reviewed release |
| **agent-detection manifest downloads** | fetched updated agent-detection rules from herdr.dev at runtime | it is a network call that changes behavior after installation. herdr-dumb uses only the manifests compiled into the binary |
| **plugins** (`herdr plugin`, plugin panes, plugin keybinding actions, the marketplace) | installed and ran third-party plugin code inside herdr | plugins are unsandboxed code from outside the reviewed source. the plugin runtime, manifest loading, and registry are deleted; plugin actions that arrive over the frozen wire protocol are rejected |
| **clipboard image forwarding** | wrote images pasted from the client to a temp directory and typed the file path into the pane | it writes outside data to disk and hands it to whatever runs in the pane. the wire message is kept for compatibility but the server ignores it |

## what stayed

local sessions that survive detaching, session restore, agent status detection (working / blocked / idle), the CLI and local socket API, keyboard and mouse control, web links, and text clipboard. panes run whatever programs you start in them with your own permissions; herdr-dumb does not sandbox them.

## differences from upstream

- the executable is `herdr-dumb` (`herdr-dumb.exe` on Windows).
- config and state live in `herdr-dumb` directories (`herdr-dumb-dev` for debug builds), so herdr-dumb never reads or changes an existing herdr install.
- the upstream install script, Homebrew, mise, and the herdr.dev docs describe the original herdr, not this fork. download herdr-dumb from this repository's [releases](https://github.com/patbonecrusher/herdr-dumb/releases), or build it yourself (see [development](#development)).
- the bundled agent guide (`src/agent-guide.md`) documents only the commands this fork has.

the rest of this README is upstream's, edited where it described removed features.

---

# herdr


<p align="center">
  <img src="assets/logo.png" alt="herdr" width="100" />
</p>

<p align="center">
  <a href="https://herdr.dev">herdr.dev</a> · <a href="#install">install</a> · <a href="https://herdr.dev/docs/quick-start/">quick start</a> · <a href="https://herdr.dev/docs/">docs</a>
</p>

<p align="center">
  English · <a href="README.zh-CN.md">简体中文</a>
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

**the runtime your coding agents live on.**

- **detach without stopping work** — herdr keeps terminals running in a background server when you close the client. after a server or machine restart, herdr restores the saved layout and can resume supported agent sessions; the original processes do not survive. [session state →](https://herdr.dev/docs/session-state/)
- **never hunt for the stuck one** — every pane is marked working, blocked, or idle. when an agent stops and needs an answer, herdr says so.
- **agent-native** — agents drive herdr through the cli and socket api: they can spawn panes, prompt each other, and wait until another agent is genuinely blocked. [agent skill →](https://herdr.dev/docs/agent-skill/)
- **runs what you already run** — claude code, codex, cursor, opencode, grok and the rest. herdr doesn't wrap or replace them; it owns their terminals.
- **keyboard and mouse, both first-class** — tmux-style prefix keys *and* click, drag, split. pick per moment, not per tool.
- **one rust binary, no electron** — runs in whatever terminal you already use.

---

## install

download the archive for your platform from [releases](https://github.com/patbonecrusher/herdr-dumb/releases):

- `herdr-dumb-windows-x86_64.zip` — keep the `conpty` directory beside `herdr-dumb.exe`
- `herdr-dumb-macos-arm64.tar.gz`

then start it where the work lives:

```bash
herdr-dumb
```

run your agents, split panes, walk away. `ctrl+b q` detaches, `herdr-dumb` reattaches. [quick start →](https://herdr.dev/docs/quick-start/)

## docs

upstream docs live at [herdr.dev/docs](https://herdr.dev/docs/) and describe the original herdr; skip anything about remote machines, updates, or plugins: [quick start](https://herdr.dev/docs/quick-start/) · [concepts](https://herdr.dev/docs/concepts/) · [supported agents](https://herdr.dev/docs/agents/) · [keyboard](https://herdr.dev/docs/keyboard/) · [configuration](https://herdr.dev/docs/configuration/) · [session state](https://herdr.dev/docs/session-state/) · [sessions](https://herdr.dev/docs/persistence-remote/) · [integrations](https://herdr.dev/docs/integrations/) · [socket api](https://herdr.dev/docs/socket-api/)

## thanks

every past sponsor and backer is listed in [SPONSORS.md](./SPONSORS.md) — thank you 🐑

enterprise / partnership: hey@herdr.dev

## agent instructions

if you are an ai agent helping with this repository, read [`AGENTS.md`](./AGENTS.md) before making changes and read [`CONTRIBUTING.md`](./CONTRIBUTING.md) before opening issues or PRs.

## development

```bash
git clone https://github.com/patbonecrusher/herdr-dumb
cd herdr-dumb
just build       # produces ./target/release/herdr-dumb

just test        # unit tests
just check       # formatting, tests, and maintenance checks
```

## license

Herdr is licensed under the [Apache License 2.0](LICENSE).
