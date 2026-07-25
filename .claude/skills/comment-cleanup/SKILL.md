---
name: comment-cleanup
description: Cleans up the comments a change touched to a terse, ASCII-only house style with no LLM tells and no header comments that relist their contents.
disable-model-invocation: true
---

# Comment Cleanup

A manual style pass over the comments in a change. Apply the rules and edit the comments in place, like an auto-formatter; don't flag them for review.

## Scope

Only touch comments added or modified in the change; leave existing comments elsewhere alone unless asked. Never change code behavior. If a comment only restates the code, delete it rather than reword it. One exception: keep comment text that renders into output, such as generated CLI help or published API docs, even when it reads like a restatement.

## Rules

1. **Terse.** Cut filler and lead-ins ("Note that", "In order to"). Keep it short but grammatical.
2. **Don't relist contents in a header.** A header comment on a method, struct, or enum that just relists its steps, fields, or variants adds nothing and rots as the code changes; delete it or keep only a non-obvious reason. Comments inside a body that describe what it does are fine. A comment that describes arguments uses an ordered list.
3. **ASCII only.** Replace smart quotes, ellipsis (…), arrows (→), and accented/emoji characters with ASCII (straight quotes, `...`, `->`). Find them with `grep -nP "[^\x00-\x7F]" <files>`.
4. **No LLM tells.**
   1. No em or en dashes (— / –), and no `-` or `--` standing in for one; restructure the sentence instead
   2. No unnecessary parentheticals
   3. No filler ("essentially", "simply", "it's worth noting")
   4. No contrast tails ("X but not Y", "X rather than Y") or negatives that add nothing
5. **Ordered lists.** Break a long series of facts into a numbered list; items carry no terminal punctuation.
6. **Periods over semicolons.** Split independent clauses into sentences. Readability beats terseness. A semicolon only separates inline list items that contain commas.

Rule 2 is the subtle one. Examples:

1. `// Loops over each user, adds the active ones to the result, and returns it` above `func ActiveUsers(...)` -> delete. Keep only a real _why_, e.g. `// Suspended accounts count as inactive.`
2. `// Config holds the host, port, timeout, and retry count` -> `// Config for the upstream connection.`
3. `// Status is one of Pending, Active, or Closed` -> `// Lifecycle state of an order.`, or delete.

Test: if deleting a comment loses nothing a reader couldn't recover from the code in seconds, delete it.
