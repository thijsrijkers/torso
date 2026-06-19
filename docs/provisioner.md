# Torso — Provisioner Design Document

> **Status:** Draft · **Version:** 0.1

---

## 1. Overview

This document describes the design of the **Provisioner**, a subsystem of the Torso infrastructure engine responsible for deploying the engine binary and its configuration to a target machine, then executing it in the correct context.

The Provisioner pushes the Torso binary and a `torso.yaml` configuration file to a remote machine and runs the binary from the directory where the configuration lives. The engine resolves its configuration by working directory convention — no flags, no paths, no ambiguity.

Torso is self-hosted by design. The Provisioner has no dependency on managed deployment platforms, cloud agents, or pre-installed tooling on the target.

---

## 2. Terminology

| Term | Definition |
|---|---|
| **Provisioner** | The subsystem that delivers and executes the Torso engine on a target machine |
| **Target machine** | The remote machine being provisioned |
| **Operator machine** | The machine initiating the provisioning |
| **torso.yaml** | The configuration file consumed by the engine at runtime |
| **Engine binary** | The compiled Torso executable |
| **Deployment directory** | The directory on the target where the binary and config are placed and executed from |

---

## 3. Goals

- Deliver the engine binary and `torso.yaml` to a target machine in a single Provisioner run
- Execute the binary with the deployment directory as working directory so `torso.yaml` is resolved automatically
- Require no pre-installed state on the target beyond SSH access and a POSIX-compatible shell
- Keep the Provisioner scriptable, repeatable, and auditable
- Support per-machine configuration without modifying the binary

---

## 4. Non-Goals

- The Provisioner does not manage secrets
- The Provisioner does not provide a GUI or web interface
- The Provisioner does not handle rollbacks in this version
- The Provisioner is not designed for mass parallel deployments across hundreds of machines simultaneously

---

## 5. Architecture

### 5.1 Components

| Component | Role |
|---|---|
| **Engine binary** | The Torso executable, compiled for the target platform architecture |
| **torso.yaml** | Per-machine configuration consumed by the engine on startup |
| **Provisioner script** | Operator-side script that drives the full provisioning sequence |
| **Target machine** | Receives the binary and config, runs the engine |

### 5.2 Deployment Directory Layout

The Provisioner places all artefacts in a single directory on the target. The binary and configuration always live together. The binary is always executed from this directory.

```
/opt/torso/
├── torso          ← engine binary
└── torso.yaml     ← configuration for this machine
```

The deployment directory path is configurable but must be consistent across machines within the same environment.

### 5.3 Provisioner Flow

The Provisioner executes the following sequence in order:

```
Operator machine
│
├─ [1] Select or generate torso.yaml for this target
├─ [2] Open SSH connection to target
├─ [3] Create deployment directory if it does not exist
├─ [4] Copy engine binary  →  /opt/torso/torso
├─ [5] Copy configuration  →  /opt/torso/torso.yaml
├─ [6] Mark binary as executable
└─ [7] Execute: cd /opt/torso && ./torso
```

Steps 2–7 are the Provisioner's responsibility. Step 1 is managed by the operator and may be automated separately via templating.

---

## 6. Technology Choices

### 6.1 Transport — SSH + SCP

**Choice:** OpenSSH (`ssh`, `scp`)

SSH is available on virtually all Linux/Unix targets without additional installation. SCP transfers files over the same authenticated channel. No agent, broker, or daemon is required on either side beyond `sshd`.

| Alternative | Reason not chosen |
|---|---|
| rsync | Useful for large file trees but adds a dependency; SCP is sufficient for two files |
| Ansible | Heavy dependency for a four-step operation; requires Python on the operator machine |
| Fabric | Adds a Python runtime dependency for what is essentially shell glue |
| Terraform / Packer | Designed for infrastructure provisioning, not binary delivery and execution |

### 6.2 Configuration Format — YAML

**Choice:** `torso.yaml`

YAML is human-readable, widely understood in infrastructure tooling, and handles hierarchical configuration cleanly. The filename is fixed — the engine always looks for `torso.yaml` in its working directory, removing any ambiguity about which configuration is active on a given machine.

| Alternative | Reason not chosen |
|---|---|
| TOML | Valid choice, but YAML is more prevalent in infrastructure tooling ecosystems |
| JSON | No comment support; harder to read and maintain by hand |
| Environment variables | Not suitable for structured, hierarchical configuration |
| CLI flags | Unmaintainable beyond trivial configs; no audit trail |

### 6.3 Provisioner Implementation — Bash

**Choice:** Bash shell script

The Provisioner's operations map directly onto shell primitives: `ssh`, `scp`, `chmod`, `cd`. A Bash script requires nothing beyond a POSIX shell on the operator machine, keeps logic transparent and auditable, and introduces no runtime dependency.

`set -euo pipefail` is enforced so the Provisioner fails hard and immediately on any error rather than continuing in a broken state.

| Alternative | Reason not chosen |
|---|---|
| Python script | Valid, but heavier for operations that are essentially shell glue |
| Makefile | Harder to parameterise cleanly for multiple target hosts |
| Ansible playbook | Overkill; introduces Ansible as an operator-side dependency |

### 6.4 Binary Distribution — Direct Push

**Choice:** SCP push from operator machine

The engine binary is compiled for the target platform and pushed directly by the Provisioner. No package manager, registry, or artefact server is required. This is the simplest and most reliable model for a self-hosted system at early scale.

As the fleet grows, a central binary store (S3-compatible object storage, internal file server, or git releases) can be introduced. The Provisioner would then instruct the target to pull the binary itself rather than pushing it.

---

## 7. Target Machine Requirements

The Provisioner makes minimal assumptions about the target. The target must have:

- An SSH server running and reachable from the operator machine
- A POSIX-compatible shell (`/bin/sh`)
- Sufficient permissions for the provisioning user to create the deployment directory and execute binaries within it
- The correct OS and CPU architecture to run the Torso binary

The engine binary is self-contained. No runtime, interpreter, or package manager is required on the target. **The Provisioner does not require Torso to be pre-installed.**

---

## 8. Configuration Management

Each target machine receives its own `torso.yaml`. Configuration is owned and managed on the operator side before provisioning runs.

- Configuration files should be stored in version control, one file per machine or per role
- Per-machine values (hostnames, IPs, environment tags) should be generated from a shared template rather than maintained as entirely separate files
- Configuration on the target must never be edited directly — the next Provisioner run will overwrite it

`torso.yaml` must not contain secrets, credentials, or tokens. Secrets are injected at runtime via environment variables or a secrets backend that the engine contacts independently.

---

## 9. Execution Model

The Provisioner executes the engine binary with the deployment directory as its working directory. This has three consequences:

- The engine resolves `torso.yaml` by working directory, no `--config` flag is needed
- Relative paths declared inside `torso.yaml` are relative to the deployment directory
- Output files, logs, and state written by the engine default to the deployment directory unless the configuration overrides them

### 9.1 Run Modes

The Provisioner supports two execution modes. The operator selects the appropriate mode per deployment.

| Mode | Behaviour |
|---|---|
| **One-shot** | The engine runs, applies the configuration, and exits. The Provisioner script is the complete execution story. |
| **Daemon** | The engine runs continuously as a managed service. A systemd unit file must be deployed alongside the binary. The Provisioner installs and starts the unit. |

Daemon mode and systemd integration are out of scope for the initial version. The deployment directory layout and execution model are explicitly designed to accommodate both modes without structural changes.

---

## 10. Security

- SSH key-based authentication is required. Password authentication must be disabled on target machines.
- The provisioning user on the target requires only write access to the deployment directory and execute permissions within it. Root access is not required.
- `torso.yaml` must never contain secrets, credentials, or API tokens.
- Binary integrity verification via checksum comparison is recommended and will be added in a future version.

---

## 11. Open Questions

| # | Question |
|---|---|---|
| 1 | Does the engine run as one-shot or daemon in production? |
| 2 | Is per-machine config generated from templates or maintained manually per machine? |
| 3 | Should the binary be versioned and stored centrally, or always pushed from a local build? |
| 4 | What is the rollback strategy when a new configuration breaks the engine? |

---

## 12. Future Considerations

- Central binary registry for version-controlled engine distribution across the fleet
- Pull-based provisioning where the target fetches its own configuration on a schedule or external trigger
- Checksum and signature verification of the binary before execution
- Provisioning inventory, tracking which machine is running which engine version and config
- Systemd unit file deployment for daemon mode support
