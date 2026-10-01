#!/bin/sh
# usage: supervisor.sh <sing-box> <config> <stop-file> <parent-pid>
# Runs sing-box until the stop file appears or the parent app exits.
bin="$1"; config="$2"; stop="$3"; parent="$4"

"$bin" run -c "$config" &
child=$!

while kill -0 "$child" 2>/dev/null && [ ! -e "$stop" ] && kill -0 "$parent" 2>/dev/null; do
  sleep 0.5
done

kill "$child" 2>/dev/null
wait "$child" 2>/dev/null
rm -f "$stop"
