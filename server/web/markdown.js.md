# markdown.js

## Function

Renders the role conversation's replies, which the model writes in
markdown, as HTML on the robot page. Without it, line breaks collapsed and
`**bold**` and lists showed as raw text (user, 2026-10-10).

## Interface

- `window.renderMarkdown(src) -> string`: HTML for a markdown string.
- Loaded by `robot.html` with a plain `<script>`; no module and no
  build step.

## Implementation

- **Escapes first, then marks up.** All HTML special characters are
  escaped before any rule runs, so the only tags in the output are the
  ones this file emits. Model text can never inject markup.
- **Supported:** paragraphs, with single newlines kept as `<br>`;
  `#`–`######` headings, shifted two levels down so they sit below the
  page's own; `-`/`*` bullets and `1.`/`1)` numbered lists, including
  indented continuation lines; fenced code blocks; `**bold**`, `*italic*`
  and `` `code` ``. Links, tables and nesting stay as plain text.
- **Hand-written instead of a library:** the page loads nothing from a
  CDN, and this covers what the replies use.

## Assertions

- Output contains no tag that the source text could have supplied. HTML
  in a reply appears as text.
- Checked by hand under node (nix-shell -p nodejs) on a sample with a
  heading, both list kinds, code and `<b>` in the text. There is no JS
  test runner in this repo.
