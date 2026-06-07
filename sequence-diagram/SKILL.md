---
name: Sequence Diagram Generator
description: Generate a Mermaid.js sequence diagram for a given function or endpoint and embed it into a Markdown file, with an example request/response and field descriptions. Use when the user wants to document or visualize the control flow of a function.
argument-hint: [function-or-file-path]
disable-model-invocation: false
---

# Sequence Diagram Generator

Generate a Mermaid.js sequence diagram documenting the implementation of a function
or endpoint, and embed it into a Markdown file.

## Target

The function, endpoint, or file to diagram: **$ARGUMENTS**

If no target was provided, ask the user which function/endpoint/file to diagram
before proceeding.

## Steps

1. **Read the actual implementation first.** Locate the target with Glob/Grep and
   Read the real source. Never diagram from assumptions — every participant,
   message, and branch must correspond to code you have actually read. Follow
   referenced helpers/queries (e.g. GraphQL files, validators) far enough to label
   messages accurately.

2. **Build the Mermaid `sequenceDiagram`.** Capture the meaningful control flow:
   participants (caller, the handler, external services like DBs/RPC/third parties),
   the sequence of calls, and decision branches (`alt`/`opt`).

3. **Embed it in a Markdown file** next to the implementation (or where the user
   asks). Wrap the diagram in a ```` ```mermaid ```` fenced code block. Open the
   file with a short title and one-paragraph description of what the function does
   and who calls it.

4. **Add an Example section** (place it before any trailing prose/notes) showing
   the **most common case**:
   - A request snippet (method, path, body) in a fenced code block.
   - A response snippet (`200` / success) in a fenced code block.
   - Use real enum values, ID formats, and field names taken from the code — not
     invented ones.

5. **Add field-description tables** for both request and response. Briefly describe
   each field. Explicitly mark a field **_(optional)_** when the code treats it as
   optional (nullable, has a default, or is only used conditionally). Note
   deprecated fields too.
   - For **nested object fields**, indent the field name with `&nbsp;&nbsp;&nbsp;&nbsp;`
     so the nesting is visually clear in the rendered table.

6. **Verify claims against the code.** Before stating that a caller "always sends"
   a parameter, confirm it in the schema/validation. If a field is optional or
   defaults to empty, represent it as optional (e.g. `field?`) in the diagram and
   tables.

## Mermaid formatting rules

- **Use `<br/>` for line breaks** inside messages and notes — **NOT `\n`**.
  Literal `\n` does not render as a newline in the VS Code Markdown/Mermaid
  previewer or on GitHub.
- Use `Note over X: ...` to summarize internal computation that isn't a call to
  another participant.
- Use `alt`/`else` for mutually exclusive branches and `opt` for conditional
  (may-or-may-not-happen) steps.

## Simplification

By default include the primary branches. If the user asks to **simplify** or
**hide error paths**, keep only the happy path: drop error/guard branches and any
participants that only appear in those paths, collapsing skipped detail into
`Note over` annotations where helpful.

## Output

After writing the file, give a one-line summary of what was created and where, and
list the branches/cases the diagram covers.
