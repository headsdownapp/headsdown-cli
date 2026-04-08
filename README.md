# HeadsDown CLI

A lightweight CLI tool (`hd`) for managing your [HeadsDown](https://headsdown.app) availability from the terminal.

## Install

### From source (Cargo)

```sh
cargo install headsdown-cli
```

### Homebrew

```sh
brew install headsdown/tap/hd
```

### Direct download

```sh
curl -fsSL https://headsdown.app/install.sh | sh
```

## Quick Start

```sh
# Authenticate (opens browser for approval)
hd auth

# Check your current status and availability
hd status
hd availability
hd windows
hd windows create --label "Focus" --mode busy --days "Mon-Fri" --start 09:00:00 --end 11:30:00
hd presets create --name "Deep Focus" --alerts do_not_disturb --presence on_keys --duration 90
hd digest list --latest 10
hd autoresponder get

# Set yourself to busy for 2 hours
hd busy 2h

# Go online
hd online

# Submit a task for verdict
hd verdict "refactor auth module" --files 5 --minutes 30

# List and activate presets
hd presets
hd preset "Focusing"

# Live dashboard with countdown
hd watch
```

## Commands

| Command | Description |
|---------|-------------|
| `hd auth` | Authenticate via Device Flow (browser-based) |
| `hd status` | Show your current availability |
| `hd availability [--at <rfc3339>]` | Show availability resolution and next transition |
| `hd windows [list]` | List configured reachability windows |
| `hd windows create ...` | Create a reachability window |
| `hd windows update <id> ...` | Update a reachability window |
| `hd windows delete <id>` | Delete a reachability window |
| `hd presets [list]` | List available presets |
| `hd presets create ...` | Create a preset |
| `hd presets update <id> ...` | Update a preset |
| `hd presets delete <id>` | Delete a preset |
| `hd preset "name"` | Activate a preset |
| `hd digest [list] [--latest N]` | List digest summaries |
| `hd digest dismiss <id>` | Dismiss a digest entry |
| `hd autoresponder get` | Show auto-responder settings |
| `hd autoresponder set ...` | Update busy/limited/offline auto-response text |
| `hd verdict-settings get` | Show verdict settings |
| `hd verdict-settings set --mode-thresholds '<json>'` | Update verdict mode thresholds |
| `hd proposals [--latest N] [--verdict approved\|deferred]` | List recent proposals |
| `hd interrupt <handle>` | Evaluate if an interrupt is allowed |
| `hd whoami` | Show your authenticated identity |
| `hd busy [duration]` | Set mode to busy |
| `hd online` | Set mode to online |
| `hd offline` | Set mode to offline |
| `hd limited [duration]` | Set mode to limited |
| `hd verdict "desc"` | Submit a task proposal and get a verdict |
| `hd watch` | Live-updating status dashboard |
| `hd doctor` | Check CLI health and connectivity |
| `hd update` | Self-update to the latest version |
| `hd hook install` | Install git hooks (auto-busy on branch switch) |
| `hd hook uninstall` | Remove git hooks |
| `hd hook status` | Show git hook status |
| `hd alias set NAME CMD` | Create a command alias |
| `hd alias remove NAME` | Remove an alias |
| `hd alias list` | List all aliases |
| `hd telemetry on\|off` | Toggle anonymous usage telemetry |
| `hd calibration on\|off\|status` | Manage calibration reporting |
| `hd outcome <proposal_id> <outcome>` | Report the real task outcome |
| `hd completions <shell>` | Generate shell completions (bash, zsh, fish) |

## Duration Formats

Durations accept human-readable formats:

- `2h` (2 hours)
- `30m` or `30min` (30 minutes)
- `1h30m` (1 hour 30 minutes)
- `1.5h` (1.5 hours = 90 minutes)
- `90` (90 minutes, bare number)
- `until 5pm` or `until 3:30pm` (until a specific time)

## JSON Output

Every command supports `--json` for machine-readable output:

```sh
hd status --json | jq .activeContract.mode
hd presets --json | jq '.[].name'
hd doctor --json
```

## Aliases

Create shortcuts for common workflows:

```sh
hd alias set focus "busy 2h"
hd alias set standup "online"
hd alias set deep "busy 4h"

# Now use them:
hd focus
hd standup
```

## Git Hooks

Auto-set your availability based on git activity:

```sh
# Install hooks in the current repo
hd hook install

# What gets installed:
# - post-checkout: auto-sets busy when switching branches
# - pre-push: auto-sets online after pushing
```

## Authentication

The CLI uses [Device Flow](https://www.rfc-editor.org/rfc/rfc8628) authentication:

1. Run `hd auth`
2. A URL and code are displayed
3. Open the URL in your browser and enter the code
4. Approve the request
5. The CLI stores your API key locally

Credentials are stored at `~/.config/headsdown/credentials` (XDG-compliant, respects `$XDG_CONFIG_HOME`).

## Configuration

Config file at `~/.config/headsdown/config.toml`:

```toml
# Default API URL
api_url = "https://headsdown.app"

# Default duration for mode commands (minutes)
default_duration = 120

# Default AI model for verdict command
default_model = "claude-sonnet-4"

[telemetry]
enabled = false

[aliases]
focus = "busy 2h"
standup = "online"
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HEADSDOWN_API_URL` | API base URL | `https://headsdown.app` |
| `XDG_CONFIG_HOME` | Config directory base | `~/.config` |
| `NO_COLOR` | Disable colored output | unset |
| `FORCE_COLOR` | Force colored output (e.g. in CI) | unset |

## Shell Completions

Generate completions for your shell:

```sh
# Bash
hd completions bash > ~/.local/share/bash-completion/completions/hd

# Zsh
hd completions zsh > ~/.zfunc/_hd

# Fish
hd completions fish > ~/.config/fish/completions/hd.fish
```

## Troubleshooting

Run the built-in diagnostic:

```sh
hd doctor
```

This checks: CLI version, config directory, credentials, API connectivity, authentication validity, and platform info.

## License

MIT
