<p align="center">
  <img src="assets/logo.svg" alt="sshmgr logo" width="780" />
</p>

<p align="center">
  <img src="https://img.shields.io/badge/rust-stable-orange" alt="Rust stable" />
  <img src="https://img.shields.io/badge/cli-clap-blue" alt="CLI clap" />
  <img src="https://img.shields.io/badge/config-serde_json-green" alt="Config serde_json" />
  <img src="https://img.shields.io/badge/license-MIT-lightgrey" alt="MIT license" />
</p>

# sshmgr

`sshmgr` is a minimal, production-ready CLI SSH session manager written in Rust.

It stores host entries in:

`~/.config/sshmgr/hosts.json`

and executes the system `ssh` command when connecting.

By default, `connect` starts a remote `tmux` session so an interactive shell can survive a dropped SSH connection. If you want the original plain SSH behavior, use `--plain`.

## Features

- Add a host entry by name
- List all saved hosts
- Connect to a saved host through a remote `tmux` session by default
- Optionally connect with plain system `ssh`
- Remove a saved host
- Graceful error handling with clear messages

## Requirements

- Linux or macOS environment with `ssh` installed
- `tmux` on the remote host for the default persistent session mode
- Rust stable toolchain (for building from source)

## Install and Build

```bash
git clone <your-repo-url>
cd sshmngr
cargo build --release
```

Built binary:

```bash
./target/release/sshmgr
```

## One-Command Install (No Rust Needed)

If Rust/Cargo is not installed, run:

```bash
git clone <your-repo-url>
cd sshmngr
./install.sh
```

This script will:

- Install Rust/Cargo if missing
- Install `tmux` if missing and a supported package manager is available
- Install `sshmgr` globally with `cargo install --path .`
- Ensure `~/.cargo/bin` is in PATH
- Print a success message

## Command Reference

### 1) Add a host

```bash
sshmgr add <name> <user> <host> <port>
```

Example:

```bash
sshmgr add webroot alice 192.168.1.10 22
```

### 2) List saved hosts

```bash
sshmgr list
```

Example output:

```text
webroot alice@192.168.1.10:22
```

### 3) Connect to host

```bash
sshmgr connect <name>
```

Example:

```bash
sshmgr connect webroot
```

This opens a remote `tmux` session by default, so the command you type stays short.

To skip `tmux` and use plain SSH, add `--plain`:

```bash
sshmgr connect --plain webroot
```

The remote host must have `tmux` installed for the default persistent mode.

### 4) Remove host

```bash
sshmgr remove <name>
```

Example:

```bash
sshmgr remove webroot
```

## Config Format

The file at `~/.config/sshmgr/hosts.json` uses this structure:

```json
{
  "hosts": [
    {
      "name": "webroot",
      "user": "alice",
      "host": "192.168.1.10",
      "port": 22
    }
  ]
}
```

## Error Handling

`sshmgr` returns user-friendly errors for common failures:

- Missing config directory detection failure
- Invalid or unreadable JSON config file
- Attempt to add a duplicate host name
- Attempt to connect/remove an unknown host
- Failure to execute `ssh`
- Non-zero exit from `ssh`

All errors are printed to stderr as:

```text
Error: <message>
```

with exit code `1`.

## Quick Run Example

```bash
./target/release/sshmgr add dbadmin ops 10.0.0.20 22
./target/release/sshmgr list
./target/release/sshmgr connect dbadmin
./target/release/sshmgr remove dbadmin
```

## Future Roadmap

The items below are intentionally unchecked and represent possible future work:

- [ ] Optional password-based auto-connect integration (with explicit security warnings)
- [ ] Import and export host lists in JSON
- [ ] Simple edit command for existing host entries
- [ ] Better list formatting options for scripts and humans
- [ ] Packaged binary releases for Linux and macOS

## Contributing

Contributions are welcome.

- Keep pull requests focused and small when possible

## License

MIT

> Star the Repository if possible!