# clat

Command line assistance tool. Describe what you want in plain English; `clat` generates a shell script and runs it.

```
clat open a port, docker pull void-base, close port
clat compress all jpegs in this directory to 80% quality
clat show disk usage sorted by size for the current directory
```

Works with any OpenAI-compatible inference API — [LM Studio](https://lmstudio.ai), Ollama, or a remote API.
Supports reasoning models (DeepSeek-R1, QwQ, etc.) — `<think>` blocks are stripped automatically.

---

## Install

### Homebrew (recommended)

```bash
brew tap OWNER/clat
brew install clat
```

### From source

```bash
make install          # builds release binary → ~/.clat/clat
```

Or manually:

```bash
cargo build --release
mkdir -p ~/.clat && cp target/release/clat ~/.clat/clat
```

Then add `~/.clat` to your `PATH` (if it isn't already):

```bash
# zsh
echo 'export PATH="$HOME/.clat:$PATH"' >> ~/.zshrc

# bash
echo 'export PATH="$HOME/.clat:$PATH"' >> ~/.bashrc
```

---

## Configuration

Config lives at `~/.clat/config.toml` — same directory as the binary.
Created automatically on first run, or explicitly with `clat --init`.

```toml
api_url           = "http://localhost:1234/v1"   # OpenAI-compatible endpoint
model             = "local-model"                # model name (see: clat -l)
api_key           = ""                           # optional bearer token
auto_run          = false                        # true = always skip confirmation
use_tools         = true                         # false for models without tool-call support
auto_run_patterns = []                           # command names that skip confirmation
                                                 # e.g. ["git", "ls", "echo"]

# Optional: override the system prompt sent with every request
# system_prompt = "..."
```

### Models

List models available from your inference server:

```bash
clat -l
# or directly:
curl http://localhost:1234/v1/models | jq '.data[].id'
```

Load a different model in LM Studio:

```bash
clat -L lmstudio-community/Meta-Llama-3-8B-Instruct-GGUF
```

### Tool calls

When `use_tools = true` (the default), `clat` sends tool definitions with each
request so the model can query the system before writing the script — for example
checking which commands are installed, or reading the current OS and working
directory. Set `use_tools = false` for models that don't support tool calling.

### sudo

Scripts that contain `sudo` are passed directly to `bash`. The OS handles the
password prompt natively — `clat` never sees your password.

---

## Usage

```
clat [OPTIONS] <prompt>...
```

| Flag | Description |
|------|-------------|
| `-y`, `--yes` | Skip confirmation, run immediately |
| `-n`, `--dry-run` | Show generated script, don't execute |
| `-l`, `--list` | List models available from the API |
| `-L`, `--load <ID>` | Load a model in LM Studio (can combine with a prompt) |
| `--model <MODEL>` | Override model for this invocation |
| `--api <URL>` | Override API URL for this invocation |
| `-v`, `--verbose` | Print prompt, API URL, model, and tool status |
| `--config` | Show current config and its path |
| `--init` | Write default config file (won't overwrite existing) |

---

## Examples

```bash
# Basic — shows generated script, asks to confirm
clat how much disk space do I have

# Skip confirmation
clat -y show my public IP address

# Dry run — see the script without executing
clat -n set up a Python venv and install requests

# List available models
clat -l

# Load a model then immediately run a prompt
clat -L lmstudio-community/Meta-Llama-3-8B-Instruct-GGUF find all TODO comments in this repo

# Override model for one call
clat --model llama-3-8b find all TODO comments in this repo

# Point at a remote API
clat --api https://api.openai.com/v1 --model gpt-4o summarise the git log

# Disable tool calls for models that don't support them
clat --api http://localhost:11434/v1 --model llama3 list open ports
# (or set use_tools = false in config)
```
