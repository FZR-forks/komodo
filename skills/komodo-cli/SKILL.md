# komodo-cli skill

## Terminal command behavior

Use `km terminal host` / `km terminal container` with `--command` (`-x`) followed by one or more command tokens.

Examples:

```bash
km terminal host server1 --command apt update
km terminal host server1 -x "apt update"
km terminal container server1 nginx -x ls -la /
```

Notes:

- `--command` accepts multiple arguments (`<COMMAND>...`).
- Host execution auto-creates the default terminal (`cli`) when needed.
- CLI output is streamed for non-interactive shells and the CLI process exits with the same code as the remote command.
