#!/bin/sh
# Waits up to nine minutes for a run to finish. Prints "done <exit code>" or
# "running <minutes> min, <passes> passes".
# Usage: wait-trial.sh <run>
root=${VOXEL_TRIALS:-$HOME/voxel-trials}
out=$root/_runs/$1
perl -e 'my ($f, $end) = (shift, time + 540); while (time < $end) { exit 0 if -e $f; select undef, undef, undef, 3 } exit 1' "$out/exit"

if [ -e "$out/exit" ]; then
  echo "done $(cat "$out/exit")"
else
  start=$(date -j -u -f %Y-%m-%dT%H:%M:%SZ "$(cat "$out/started")" +%s)
  passes=$(ls -d "$out"/passes/[0-9]* 2> /dev/null | wc -l | tr -d ' ')
  echo "running $((($(date +%s) - start) / 60)) min, $passes passes"
fi
