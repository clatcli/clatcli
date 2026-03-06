# clat

Command line assistance tool. Describe what you want in plain English; `clat` generates a shell script and runs it.

```
clat open a port, docker pull void-base, close port
clat compress all jpegs in this directory to 80% quality
clat show disk usage sorted by size for the current directory
```

Works with any OpenAI-compatible inference API — [LM Studio](https://lmstudio.ai), Ollama, or a remote API.

---

## Install

```bash
make install          # builds release binary → /usr/local/bin/clat
```

Or manually:

```bash
cargo build --release
cp target/release/clat /usr/local/bin/clat
```

---

## Configuration

Config file is created automatically on first run. To initialise it explicitly:

```bash
clat --init
```

**Location:**
- macOS: `~/Library/Application Support/clat/config.toml`
- Linux: `~/.config/clat/config.toml`

**Options:**

```toml
api_url    = "http://localhost:1234/v1"   # OpenAI-compatible endpoint
model      = "local-model"                # model name (check /v1/models)
api_key    = ""                           # optional bearer token
auto_run   = false                        # true = skip confirmation prompt

# Optional: override the system prompt
# system_prompt = "..."
```

To find the right model name when using LM Studio:

```bash
curl http://localhost:1234/v1/models | jq '.data[].id'
```

---

## Usage

```
clat [OPTIONS] <prompt>...
```

| Flag | Description |
|------|-------------|
| `-y`, `--yes` | Skip confirmation, run immediately |
| `-n`, `--dry-run` | Show generated script, don't execute |
| `--model <MODEL>` | Override model for this invocation |
| `--api <URL>` | Override API URL for this invocation |
| `-v`, `--verbose` | Print prompt, API URL, and model before calling |
| `--config` | Show current config and its path |
| `--init` | Write default config file (won't overwrite) |

---

## Examples

```bash
# Basic usage — shows script, asks to confirm
clat list the 10 largest files under /var/log

# Skip confirmation
clat -y show my public IP address

# Dry run — see the script without executing
clat -n set up a Python venv and install requests

# Use a different model for one call
clat --model llama-3-8b find all TODO comments in this repo

# Point at a remote API
clat --api https://api.openai.com/v1 --model gpt-4o summarise the git log
```
