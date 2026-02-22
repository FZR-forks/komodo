# komodo-cli skill

## Terminal command behavior

Use `km terminal host` / `km terminal container` with `--command` (`-x`) followed by one or more command tokens.

## Examples

```bash
# host terminal
km terminal host server1 --command apt update
km terminal host server1 -x "apt update"
# quotes are optional for simple multi-token commands, but recommended when args contain spaces/shell chars
km terminal host server1 --command "cat '/var/log/my file.log' | grep foo"

# container terminal
km terminal container server1 nginx --command ls -la /
km terminal container server1 redis -s bash --command printenv
```

## Notes

- `--command` accepts multiple arguments (`<COMMAND>...`) so quoting is optional for simple commands, but use quotes when arguments include spaces or shell-special characters (pipes, redirects, globbing).
- In `km terminal container`, `-s` / `--shell` selects the shell used for `--command` execution (for example, `-s bash`).
- `km terminal host` automatically retries once by creating the requested terminal if the initial execute call fails with `No terminal at <name>`.
- CLI streams output line-by-line (including non-interactive shells) and exits with the remote command exit code.
