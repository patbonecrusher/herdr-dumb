# herdr-dumb agent guide

This fork manages local terminal sessions and coding agents. The executable is
`herdr-dumb`, not `herdr`. Use `herdr-dumb --help` and per-command `--help` for
the commands available in this build.

## Local control

Use control commands only when the user has asked you to manage Herdr. Check
`HERDR_ENV` before controlling panes from inside a managed session, and use
`--help` for discovery rather than launching the bare executable. Do not send
input to an unrelated session or pane.

The CLI talks to the local server through a Unix socket or Windows named pipe.
There is no built-in SSH attachment, saved-machine routing, network control
listener, or reverse tunnel. Do not use `--remote`, `--machine`, or `machine`.

Useful discovery commands:

```sh
herdr-dumb status
herdr-dumb workspace list
herdr-dumb pane list
herdr-dumb agent list
herdr-dumb api schema --json
```

Use returned workspace, tab, pane, and terminal IDs rather than guessing them.
Read a pane before sending input; check the command's help for its target and
input options. JSON output is intended for automation. Agent detection reads
the detection buffer, which is independent of a user's scrolled viewport.

```sh
herdr-dumb agent read <pane> --source detection --format text
herdr-dumb agent explain <pane> --json
```

## Sessions and configuration

```sh
herdr-dumb --session work
herdr-dumb session list
herdr-dumb config check
herdr-dumb --default-config
```

Config and state directories use `herdr-dumb` (`herdr-dumb-dev` in debug builds),
separate from upstream Herdr. The existing `HERDR_*` environment contract is
retained for agent/plugin compatibility. Explicit `HERDR_SOCKET_PATH` and
`HERDR_CLIENT_SOCKET_PATH` overrides take precedence; clear them to avoid
targeting another installation. `HERDR_BIN_PATH` identifies this executable
inside managed panes and is the preferred executable path for hooks.

## Updates and plugins

Application updates, update channels, and detection-manifest downloads are
removed. Do not run `update`, `channel`, or `server update-agent-manifests`.
Build manually with `just build` to produce `target/release/herdr-dumb`.
Bundled detection rules and local overrides remain supported; use
`herdr-dumb server reload-agent-manifests` after editing a local override.

Plugin installation/downloads, local agent integrations, opening web links,
and local clipboard operations remain available. Inspect `plugin --help` and
`integration --help` before changing installations. Plugins and pane programs
run with the user's privileges and are not sandboxed; only run trusted code.
