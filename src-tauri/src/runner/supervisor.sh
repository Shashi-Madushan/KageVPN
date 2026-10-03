#!/bin/sh
# usage: supervisor.sh <sing-box> <config> <stop-file> <parent-pid> [kill-switch-ips]
# Runs sing-box until the stop file appears or the parent app exits. When a
# kill-switch IP list is given (even "-"), traffic is blocked if sing-box dies
# on its own, until the stop file appears or the parent exits.
bin="$1"; config="$2"; stop="$3"; parent="$4"; ips="$5"
anchor="com.apple/250.OspreyVPN"
token=""

engage_block() {
  list=$(echo "$ips" | tr ',' ' ')
  [ "$list" = "-" ] && list=""
  {
    echo "pass quick on lo0 all"
    echo "pass out quick to { 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16 169.254.0.0/16 224.0.0.0/4 255.255.255.255 }"
    [ -n "$list" ] && echo "pass out quick to { $list }"
    echo "block drop out all"
  } | pfctl -a "$anchor" -f - 2>/dev/null
  token=$(pfctl -E 2>&1 | sed -n 's/.*Token : //p')
}

release_block() {
  pfctl -a "$anchor" -F all 2>/dev/null
  [ -n "$token" ] && pfctl -X "$token" 2>/dev/null
  token=""
}

"$bin" run -c "$config" &
child=$!

while kill -0 "$child" 2>/dev/null && [ ! -e "$stop" ] && kill -0 "$parent" 2>/dev/null; do
  sleep 0.5
done

if [ -n "$ips" ] && [ ! -e "$stop" ] && kill -0 "$parent" 2>/dev/null && ! kill -0 "$child" 2>/dev/null; then
  engage_block
  while [ ! -e "$stop" ] && kill -0 "$parent" 2>/dev/null; do
    sleep 0.5
  done
  release_block
fi

kill "$child" 2>/dev/null
wait "$child" 2>/dev/null
rm -f "$stop"
