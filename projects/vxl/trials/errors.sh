#!/bin/sh
# Prints each failed tool call in a run's transcript with the command before it.
# Usage: errors.sh [--round <round>] <run>
here=$(cd "$(dirname "$0")" && pwd)
python3 "$here/summarize-transcript.py" "$@" 600 | awk '
  /^## TOOL / { tool = $0; getline cmd; next }
  /^## RESULT ERROR/ {
    print "--", tool ": " substr(cmd, 1, 160)
    getline line
    for (i = 0; i < 4 && line != ""; i++) { print "   " line; getline line }
  }'
