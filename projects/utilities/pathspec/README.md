# pathspec

Gitignore-style glob pattern matching over logical paths.

Patterns follow `.gitignore` rules and match against logical paths, strings
whose segments are separated by `/`. A path need not exist on disk; the caller
says whether each path is a directory or a leaf. It is built on the `globset`
crate, so pattern compilation and escaping match real gitignore rather than a
hand-written regex.

## Pattern syntax

A plain pattern includes; a leading `!` excludes. A trailing `/` matches
directories only. A leading or interior `/` anchors the pattern to the root; a
pattern with no `/` floats and matches at any depth. `*` and `?` match within a
segment, `**` crosses segments, and `[...]` is a character class. A blank line
or a line starting with `#` is inert.

Across an ordered list of patterns the last one to match a path decides the
outcome, so a later pattern can override an earlier one.
