# Terminals

Komodo provides browser-based terminal sessions for servers and containers. Sessions are persistent, support multiple simultaneous connections, and commands can be scripted and scheduled.

In this forked CLI, terminal behavior is exposed as:

- `km ssh host ...` for interactive host terminals
- `km ssh container ...` for interactive container terminals
- `km exec host ... -x '...'` for one-shot host commands
- `km exec container ... -x '...'` for one-shot container commands
- `km attach ...` for interactive container attach sessions

There is no public top-level `km terminal` command in this fork.

## Server Terminals

Open a shell directly on a connected server. The default command is `bash`, configurable per Periphery via `default_terminal_command`.

CLI examples:

```sh
km ssh host my-server
km ssh host my-server -t maintenance
km exec host my-server -x 'df -h'
```

## Container Terminals

Connect to a running container in two modes:

- Exec: runs a new command inside the container, similar to `docker exec`
- Attach: attaches to the container's main process, similar to `docker attach`

Container terminals are available on deployments, stack services, and any container visible on a server.

CLI examples:

```sh
km ssh container nginx --server my-server
km exec container nginx --server my-server -x 'printenv'
km attach nginx --server my-server
```

The `-x/--command` flag intentionally accepts a single remote command string. This avoids local shell interpretation problems that argv-style forwarding does not solve.

## Multiple Sessions

You can create multiple named terminal sessions on the same resource. Each session has its own PTY process and output history.

- Terminal names must be unique within a target.
- Multiple users can connect to the same terminal session simultaneously.
- Sessions persist until explicitly deleted or Periphery restarts.

## Terminal History

Each terminal maintains a rolling 1 MiB output buffer. When you reconnect to an existing session, the history is replayed.

## Execute Terminal API

The `execute_terminal` API method runs a command on a terminal and streams output back over HTTP. This is useful for:

- Actions
- Automation
- External tooling that talks to the REST API directly

Upstream v2 uses a generic request shape built around `target`, `terminal`, `command`, and `init`. This fork's CLI commands map onto that API rather than the old pre-v2 terminal execution shape.

The TypeScript client provides convenience methods for each target type. All methods accept optional callbacks with `onLine` and `onFinish`.

```typescript
await komodo.execute_server_terminal(
  {
    server: "my-server",
    terminal: "automation",
    command: "df -h",
    init: { command: "bash", recreate: "DifferentCommand" },
  },
  {
    onLine: (line) => console.log(line),
    onFinish: (code) => console.log("Exit code:", code),
  },
)

await komodo.execute_container_terminal(
  {
    server: "my-server",
    container: "my-container",
    terminal: "debug",
    command: "cat /var/log/errors.log",
    init: { command: "sh", mode: "Exec", recreate: "Never" },
  },
  {
    onLine: (line) => console.log(line),
    onFinish: (code) => console.log("Exit code:", code),
  },
)

await komodo.execute_stack_service_terminal(
  {
    stack: "my-stack",
    service: "web",
    terminal: "debug",
    command: "nginx -t",
    init: { command: "sh" },
  },
  {
    onLine: (line) => console.log(line),
    onFinish: (code) => console.log("Exit code:", code),
  },
)

await komodo.execute_deployment_terminal(
  {
    deployment: "my-deployment",
    terminal: "check",
    command: "node --version",
    init: { command: "sh", recreate: "Always" },
  },
  {
    onLine: (line) => console.log(line),
    onFinish: (code) => console.log("Exit code:", code),
  },
)
```

## Periphery Configuration

Terminal behavior can be configured in the Periphery config file:

| Setting | Description | Default |
|---|---|---|
| `default_terminal_command` | Default shell command for new server terminals. | `bash` |
| `disable_terminals` | Disable server terminal sessions. | `false` |
| `disable_container_terminals` | Disable container terminal sessions. | `false` |
