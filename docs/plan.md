# FEATURES

## ✅ PoC Codex
- Assumes the Codex CLI is already installed
- A simple wrapper between the user and the Codex CLI
- Shell command `ai-usage` returns the current Codex CLI `/status` response as-is

## ✅ PoC Claude
- Same as above, but for the Claude CLI

## ❌ PoC Cursor
- Same as above for Cursor, but that approach did not work; we found a workaround via an undocumented API

## ✅ MVP
- Supports Claude, Codex, and Cursor
- Assumes the Claude and Codex CLIs are already installed
- Shell command `ai-usage`

## Post-MVP features

- Daemon for regular polling
   - Configurable polling frequency
   - Explicit command to start the daemon
   - Explicit command to stop the daemon
- "From scratch" workflow when the user has no CLI
   - For Claude, Codex, and Cursor
   - Should be solvable in one command: install the CLI
   - Corresponding documentation for people and agents (En, Ru)
- Native command invocation from the device
   - Command available from any directory
   - Onboarding flow that feels natural to the user
- Mac
   - Mac desktop app (Tauri?)
   - Mac widgets
   - System notifications
      - Thresholds for triggering notifications
      - Default notification set
      - Explicit flow for the user to enable notifications
      - Configuration via terminal
      - Configuration via config file
      - Configuration via desktop app UI
- Hard stop on spending when limits are reached
- Analytics
   - Minimal database (SQLite?)
   - Comparison of the two most recent requests
   - Trends in prettified format: whether token usage went up or down, and by how much
   - Simple charts
   - Manual database cleanup
   - Cleanup by limits (period or count)
   - Desktop interface
   - Self-analysis of this tool: how many tokens `ai-usage-mit` itself consumes
   - Later, decide what else to analyze
      - By provider
      - By model
- Via API
   - Codex
   - Claude
   - Cursor
- Other installation methods
   - NPM?
   - Pip?
- Windows
   - Desktop app
   - System notifications
   - Widgets
- Linux
   - Desktop app
   - System notifications 
- Command-line tools
   - Output flags `prettified` and `raw`
   - `prettified` as the default format
   - Nicer `prettified` format
   - Terminal loader while the command runs
   - Multiple subscriptions for one provider
   - Analytics from another device
   - Tunnel?

- More providers
   - OpenAI API.
   - ChatGPT subscription usage, if data is available locally or via export.
   - Anthropic API.
   - Claude Code.
   - Google Gemini API.
   - Gemini CLI.
   - xAI API.
   - Mistral API.
   - Groq API.
   - OpenRouter.
   - Perplexity API.
   - Cohere.
   - Together AI.
   - Fireworks AI.
   - DeepSeek.
   - Ollama.
   - LM Studio.
   - LocalAI.
   - Azure OpenAI.
   - AWS Bedrock.
   - Google Vertex AI.
   - Cursor.
   - GitHub Copilot CLI/agentic tools, if local stats are available.
   - Codex CLI.
   - Aider.
   - Continue.
   - Cline/Roo Code.
   - OpenCode.
   - LiteLLM logs.
   - LangChain/LangSmith exports, if import is needed.
   - Custom provider via adapter.
