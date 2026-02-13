# Komodo CLI

Komodo CLI is a tool to execute actions on your Komodo instance from shell scripts.

## Install

```sh
cargo install komodo_cli
```

Note: On Ubuntu, also requires `apt install build-essential pkg-config libssl-dev`.

## Usage

### Credentials

Configure credentials using environment variables:
```sh
export KM_KOMODO_URL="https://your.komodo.address"
export KM_KOMODO_API_KEY="YOUR-API-KEY"
export KM_KOMODO_API_SECRET="YOUR-API-SECRET"
```

Or configure a file `~/.config/komodo/komodo.cli.toml` with contents:
```toml
host = "https://your.komodo.address"
cli_key = "YOUR-API-KEY"
cli_secret = "YOUR-API-SECRET"
```

You can also pass the information using command line arguments:
```sh
km execute -a "https://your.komodo.address" -k "YOUR-API-KEY" -s "YOUR-API-SECRET" ...
```

### Listing Resources

List all resources or filter by type using `km list`:
```sh
km list                          # List all resources
km list stacks                   # List all stacks
km list procedures               # List all procedures
km list syncs                    # List all resource syncs
km list -d                       # List only down/failed resources
km list -a                       # List all including down
```

### Stack Operations

Get detailed information about a specific stack:
```sh
km stack my-stack status         # Get detailed status
km stk my-stack s                # Using aliases
```

View stack container logs:
```sh
km stack my-stack logs                    # View container logs (last 100 lines)
km stack my-stack logs -s nginx           # Filter to specific service
km stack my-stack logs -s nginx -s redis  # Multiple services
km stack my-stack logs -n 200             # Show last 200 lines (default: 100, max: 5000)
km stack my-stack logs -t                 # Include timestamps
```

List services in a stack:
```sh
km stack my-stack services       # List all services and their state
```

View deployment history and detailed logs:
```sh
km stack my-stack deploys              # Show most recent 10 deployments
km stack my-stack deploys -n 20        # Show most recent 20 deployments
km stack my-stack deploy-log <ID>      # View detailed logs for a specific deployment
```

### Procedure Operations

Get information about a procedure:
```sh
km procedure my-proc status            # Get detailed status
km proc my-proc s                      # Using aliases
```

View run history and detailed logs:
```sh
km procedure my-proc logs              # Show most recent 10 runs
km procedure my-proc logs -n 20        # Show most recent 20 runs
km procedure my-proc run-log <ID>      # View detailed logs for a specific run
```

### Resource Sync Operations

Get information about a sync:
```sh
km sync my-sync status                 # Get detailed status
km sn my-sync s                        # Using aliases
km sync my-sync diff                   # Show pending diffs from upstream
```

View run history and detailed logs:
```sh
km sync my-sync logs                   # Show most recent 10 runs
km sync my-sync logs -n 20             # Show most recent 20 runs
km sync my-sync run-log <ID>           # View detailed logs for a specific run
```

### Variable Management

```sh
km variable list                 # List all variables (secrets shown as ********)
km var ls                        # Using aliases
km var get MY_VAR                # Get variable value (requires KM_SHOW_SECRETS=true for secrets)
```

Create a variable:
```sh
km var create MY_VAR "my-value"              # Create regular variable
km var create API_KEY "secret" -s            # Create secret variable
km var create API_KEY "secret" -s -d "desc"  # With description
```

Create a variable from command output:
```sh
km var create API_KEY -c "openssl rand -hex 32" -s  # Value from command
```

Delete a variable:
```sh
km var delete MY_VAR             # Delete with confirmation
km var delete MY_VAR -y          # Delete without confirmation
```

### Run Executions

Execute commands require confirmation by default. Use `-y` or `--yes` to skip confirmation for automation/scripting:

```sh
# Interactive (requires pressing ENTER)
km execute deploy-stack my-stack
km execute run-sync my-sync
km execute run-procedure my-procedure

# Non-interactive (for scripts/automation)
km execute deploy-stack my-stack -y
km execute run-sync my-sync --yes
km execute run-procedure my-procedure -y
```

Other execution examples:
```sh
km execute run-build test_build -y       # Run a build
km execute destroy-stack my-stack -y     # Destroy a stack
```

### Terminal Commands

Run remote terminal commands on hosts and containers:
```sh
# Execute on a server terminal (terminal name defaults to "cli")
km terminal host my-server -x "uname -a"
km term host my-server -t maintenance -x "df -h"   # custom terminal name + alias

# Execute inside a container shell
km terminal container my-server nginx -x "ls -la /"
km tm c my-server redis -s bash -x "printenv"       # alias + custom shell
```

The CLI streams command output directly and returns a non-zero exit if the remote command exits non-zero.

### Other Commands

```sh
km config                        # Print the CLI config being used
km container                     # Container info
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `KM_KOMODO_URL` | Komodo server URL (alias: `KOMODO_CLI_HOST`) |
| `KM_KOMODO_API_KEY` | API key (alias: `KOMODO_CLI_KEY`) |
| `KM_KOMODO_API_SECRET` | API secret (alias: `KOMODO_CLI_SECRET`) |
| `KM_SHOW_SECRETS` | Set to `true` to allow viewing secret variable values |

## Full Command Reference

`km --help`
```md
Usage: km [OPTIONS] <COMMAND>

Commands:
  config     Print the CLI config being used
  container  Container info
  inspect    Inspect containers
  list       List Komodo resources
  execute    Run Komodo executions
  update     Update resource configuration
  database   Database utilities
  stack      Stack operations (status, logs, services, deploys)
  variable   Variable management (list, get, create, delete)
  procedure  Procedure operations (status, logs, run-log)
  sync       Resource sync operations (status, logs, run-log, diff)
  terminal   Terminal commands on hosts and containers
  help       Print this message or the help of the given subcommand(s)

Options:
  -p, --profile <PROFILE>          Choose a custom profile
  -c, --config-path <CONFIG_PATH>  Config file or directory path
  -h, --help                       Print help
  -V, --version                    Print version
```

