# Contract: Shell Hook Behavior

## Supported Shells

- Zsh for the first interactive hook implementation.
- Bash compatibility validation is required for release; Bash hook behavior may
  be implemented in a separate phase but must preserve the same user contract.

## Rendering Contract

- Render at most one suggestion.
- Suggestion appears as gray Ghost Text to the right of the cursor.
- Suggestion is a suffix only; it must not alter the user's typed text until
  accepted.
- No multiline rendering, popup UI, command palette, Markdown, explanatory text,
  or chat surface is allowed.

## Keystroke Contract

- `Tab` accepts the visible suggestion when one is active.
- `Right Arrow` accepts the visible suggestion when one is active.
- If no suggestion is active, `Tab`, `Right Arrow`, and unrelated shortcuts must
  fall through to native shell behavior.
- Continuing to type clears or replaces stale suggestion state.

## Request Contract

- Hook observes command-line changes and applies a 30ms debounce before sending a
  completion request.
- Hook includes current line, cursor, cwd, shell kind, session id, request id, and
  deadline.
- Hook never blocks the shell main input path waiting for daemon work.
- Hook silently no-ops if daemon probe or socket connection fails.

## Stale Response Contract

- Hook renders a response only if response id matches the latest active request.
- Hook renders a response only if line/cursor state still matches the request
  state.
- Hook ignores responses that arrive after the user has typed additional input.

## Failure Contract

- No daemon/socket/model/database failure may print output into the terminal.
- Failure outcome is always native shell behavior with no suggestion.
