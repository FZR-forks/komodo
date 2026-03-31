---
name: komodo-cli
description: Use when the user asks about managing their homelab, deployments, services, containers, stacks, servers, SSH sessions, logs, builds, variables, procedures, syncs, or anything related to the Komodo infrastructure management CLI (`km`). Triggers on keywords like deploy, restart, logs, status, SSH, container, stack, server, build, variable, procedure, sync, homelab, infrastructure.
---

# Komodo CLI (`km`)

You are assisting a user who manages their homelab infrastructure through the Komodo CLI (`km`). This CLI is a thin client that communicates with a Komodo Core backend server over HTTP REST and WebSocket APIs.

## Architecture

- **`km`** is the CLI binary (Rust, async/tokio)
- All state lives on the **Komodo Core** server — the CLI is stateless


# Komodo CLI

Komodo CLI is a tool to execute actions on your Komodo instance from shell scripts.

## Usage

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
km stack my-stack logs                    # View container logs
km stack my-stack logs -s nginx           # Filter to specific service
km stack my-stack logs -s nginx -s redis  # Multiple services
km stack my-stack logs -n 200 -t          # 200 lines with timestamps
```

List services in a stack:
```sh
km stack my-stack services       # List all services and their state
```

View deployment history:
```sh
km stack my-stack deploys        # Show deployment history
km stack my-stack deploys -n 20  # Show last 20 deployments
km stack my-stack deploy-log <update-id>  # View logs for a specific deployment
```

### Procedure Operations

Get information about a procedure:
```sh
km procedure my-proc status      # Get detailed status
km proc my-proc s                # Using aliases
km procedure my-proc logs        # View run history
km procedure my-proc logs -n 20  # Show last 20 runs
```

### Resource Sync Operations

Get information about a sync:
```sh
km sync my-sync status           # Get detailed status
km sn my-sync s                  # Using aliases
km sync my-sync logs             # View sync run history
km sync my-sync diff             # Show pending diffs from upstream
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

### Container lifecycle
```bash
km x start-container my-server my-app
km x restart-container my-server my-app
km x stop-container my-server my-app
km x destroy-container my-server my-app
km x prune-containers my-server
```

### Inspect containers
```bash
km inspect my-container                  # Full container inspect
km inspect my-container -u               # State only
km inspect my-container -m               # Mounts only
km inspect my-container -n               # Network settings only
```

### Update resource configs
```bash
km update deployment my-dep "image=nginx:latest"
km update stack my-stack "run_directory=/opt/stacks/my-stack"
km update server my-server "address=192.168.1.100"
km update variable MY_VAR "new-value"
```

### Run Executions

```sh
# Triggers an example build
km execute run-build test_build -y

# Deploy a stack
km execute deploy-stack my-stack -y

# Run a procedure
km execute run-procedure my-procedure -y

# Run a sync
km execute run-sync my-sync -y
```

### SSH and remote commands
```bash
km ssh host my-server            # Interactive SSH to server
km ssh container my-container    # Interactive shell in container
km exec host my-server -x "df -h"           # One-shot command on server
km exec container my-app --server my-server -x "ls /data"  # One-shot command in container
km attach my-container --server my-server    # Attach to container terminal
```

The CLI streams command output directly and returns a non-zero exit if the remote command exits non-zero

## Key Patterns

- **Wildcards**: Many list/batch commands support wildcard patterns (`*`, `?`) for filtering by name
- **Aliases**: Most commands have short aliases (e.g., `ls` for `list`, `x` for `execute`, `stk` for `stack`, `ps` for `container`)
- **`-y` flag**: Skip confirmation prompts for automation/scripting
- **`-f json`**: Get JSON output instead of tables for scripting
- **Profiles**: Use `-p profile-name` to switch between different Komodo instances
- **Batch operations**: Use kebab-case batch execution commands (e.g., `batch-deploy`, `batch-destroy-stack`)

## Guidance

- When the user asks to "check on" or "see status of" something, start with `km ls` or `km stk <name> status`
- When they want to "restart" or "redeploy", use the appropriate `km x` execution command
- For debugging, suggest checking logs first: `km stk <name> logs` or `km ps -d` for down containers
- Always prefer the short aliases when suggesting commands to keep things concise
- If the user mentions a specific server, use the `-s` filter flag to scope commands
- For secret values, remind them about `KM_SHOW_SECRETS=true` if they need to see actual values
