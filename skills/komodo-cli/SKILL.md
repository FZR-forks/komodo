---
name: komodo-cli
description: Access and manage variables, stacks and resources syncs in Komodo deployment system for my server network. See the logs of Stacks, services or komodo Procedure or resource sync runs.
compatibility: opencode
---

## What I do

- Access deployment status for stacks
- Show the logs of stacks and services in those stacks
- show the logs and status of resource sync (runs) and procedure (runs)
- List, add or remove variables, with the ability of setting the value as the output of a command, perfect for secrets.
- run shell commands on the server or inside containers.

## When to use me

When debugging issues with Komodo, 
Examples:

- Why the deployment is failing after adding a new service (read the logs of the last deployment run for the stack)
- Why the changes to the configuration aren't applying (Check the logs of the last procedure and resource sync runs)
- Why is the service still not working (Check the stack logs or the logs of one container in the stack)

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

### Run Executions

```sh
# Triggers an example build
km execute run-build test_build

# Deploy a stack
km execute deploy-stack my-stack

# Run a procedure
km execute run-procedure my-procedure

# Run a sync
km execute run-sync my-sync
```

### Shell Commands

Run remote commands with `bash` or `sh` on hosts and containers:
```sh
# Execute on a host
km bash host my-server -x "uname -a"
km sh host my-server -x "df -h"

# Execute inside a container
km bash container my-server nginx -x "ls -la /"
km sh container my-server caddy -x "printenv"
```

Use quotes around `--command` (recommended for all usage), especially for shell operators and multiline commands:
```sh
km bash host my-server -x "set -e; whoami; id"
km sh container my-server app -x $'echo start\nid\necho done'
```

Behavior notes:
- `--command` / `-x` is required.
- Host commands run in a temporary terminal that is created and deleted per command.
- `km bash container ...` automatically retries with `sh` if `bash` is unavailable (common in Alpine images).
- The CLI streams output directly and exits with the same status code as the remote command.
