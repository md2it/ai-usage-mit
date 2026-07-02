# Claude Code statusline setup for ai-limits

Give this prompt to Claude Code once on the machine where `ai-limits` is installed:

```text
Configure Claude Code statusLine.command so it runs:

ai-limits --claude-statusline

Use the correct Claude Code settings file for this machine. Do not remove existing settings. If a statusLine command already exists, preserve it when possible or explain the conflict before changing anything.

After the setup, I will send one normal Claude Code request so ai-limits can capture the latest rate_limits payload.
```

After Claude Code updates its settings, send any normal Claude Code request once. Then run `ai-limits` again.

This setup enables Claude Code live limits/reset through `claude_statusline_rate_limits`. It does not confirm coverage for Claude Desktop, Claude web, or browser-extension usage.
