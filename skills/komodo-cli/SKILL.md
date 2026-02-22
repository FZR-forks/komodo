# komodo-cli skill

## Terminal command behavior

Use `km terminal host` / `km terminal container` with `--command` (`-x`) followed by one or more command tokens.

## Examples

```bash
# host terminal
km terminal host server1 --command apt update
km terminal host server1 -x "apt update"

# container terminal
km terminal container server1 nginx --command ls -la /
km terminal container server1 redis -s bash -x printenv
```

## Notes

- `--command` accepts multiple arguments (`<COMMAND>...`) so quoting is optional.
- `km terminal host` automatically retries once by creating the requested terminal if the initial execute call fails with `No terminal at <name>`.
- CLI streams output line-by-line (including non-interactive shells) and exits with the remote command exit code.
