
# Docker Deployment Module
 
> **Scope:** This module receives a deployment config from the engine's API layer, pulls the described image from a registry, runs it as a container, writes logs to disk, and rolls back automatically if the new deployment fails. It owns nothing outside of that, no HTTP, no global state, no routing.
 
---
 
## Responsibility
 
The module takes a deployment config, talks to the local Docker daemon over the Unix socket, and returns either a running container result or an error. Logs are written to disk as a side effect. Everything else, HTTP, state, routing, is the API layer's responsibility.
 
---
 
## Config
 
The module expects a resolved config, secrets are substituted upstream, no `${{ }}` placeholders reach this module. The config describes the image to pull, the registry to pull from (if private), the host and port to bind, environment variables, volume mounts, and restart and pull policies.
 
`PullPolicy` controls whether the module skips the pull if the image tag is already present locally. `Always` is the safe default, `IfNotPresent` risks stale images for mutable tags like `latest`.
 
---
 
## Deployment Flow
 
Steps run in order. Any failure aborts unless noted otherwise.
 
1. **Read rollback state** — load the previous image tag from disk if one exists.
2. **Authenticate** — build registry credentials from config. Skipped for public images.
3. **Pull image** — stream the image to the local daemon. Progress is written to the engine log.
4. **Stop existing container** — graceful stop by app name. Silently ignored if nothing is running.
5. **Remove existing container** — force remove by app name. Silently ignored if nothing exists.
6. **Create container** — apply all config: ports, env vars, volumes, restart policy.
7. **Start container** — on success, write new rollback state and begin log streaming. On failure, trigger rollback.
Steps 4 and 5 are the only steps that continue past a failure. Everything else aborts and returns an error.
 
---
 
## Rollback
 
Rollback is intentionally minimal. The module only rolls back one step, from a failed new deploy back to the last known-good image.
 
Before replacing a running container the module writes the current image tag to a small file on disk. If the new container fails to start, that file is read and the previous image is deployed in its place.
 
If rollback succeeds the module returns a success result with `rolled_back: true`, something is running, but it is not what the caller asked for. If rollback also fails the module returns a hard error.
 
Rollback does not recurse or retry. It is best-effort, one attempt.
 
---
 
## Return Value
 
On success the module returns:
 
- Container ID
- App name
- Image that is actually running
- Host and port it is bound to
- `rolled_back` flag
**The `rolled_back` flag** signals a specific third state: the new deploy failed at start, the engine recovered by restarting the previous image, and the machine is stable, but the caller's requested version is not running. This is meaningfully different from both a clean success and a hard failure. The API layer should surface this distinction to the caller rather than collapsing it into a generic error, the system is healthy, but the deploy did not take effect.
 
---
 
## Errors
 
Each error maps to exactly one step so the API layer can give the caller a specific failure reason:
 
- **Connection** — could not reach the Docker daemon
- **Pull** — image pull failed
- **Stop** — failed to stop existing container
- **Remove** — failed to remove existing container
- **Create** — failed to create new container
- **StartAndRollbackFailed** — container failed to start and rollback also failed
- **Io** — could not write logs or rollback state to disk
`StartAndRollbackFailed` is only returned when both the deploy and the rollback failed. If rollback succeeded, the function returns a success result with `rolled_back: true`.
 
---
 
## Logs
 
Two log files per app, written to a configurable base directory:
 
**`container.log`** — stdout and stderr from the running container, streamed from the Docker daemon with timestamps. Written as a background task that starts after the container starts. If the container stops and restarts due to its restart policy, the stream is re-attached.
 
**`engine.log`** — structured plain-text events from the module itself. One line per event covering: pull started, pull complete, container stopped, container started, start failed, rollback triggered, rollback complete. Enough to reconstruct exactly what happened and when.
 
---
 
## Public Interface
 
Two functions:
 
**`deploy(config, log_dir)`** — runs the full deployment flow. Returns the container result or an error. Log directory is passed in so the module does not decide where logs live.
 
**`remove(name)`** — stops and removes a container by name. Idempotent does not error if the container does not exist.
 
---
 
## Dependencies
 
| Crate | Purpose |
|-------|---------|
| `bollard` | Docker daemon client over Unix socket |
| `tokio` | Async runtime |
| `futures-util` | Stream processing for pull progress and log streaming |
| `tracing` + `tracing-appender` | Engine event logging to file |
| `thiserror` | Structured error types |
| `serde` / `serde_yaml` | Config deserialization |
 
