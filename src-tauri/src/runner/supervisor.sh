#!/bin/sh
# usage: supervisor.sh <sing-box> <config> <stop-file> <parent-pid> [kill-switch-ips]
# Runs sing-box until the stop file appears or the parent app exits. When a
# kill-switch IP list is given (even "-"), traffic is blocked if sing-box dies
# on its own, until the stop file appears or the parent exits.
bin="$1"; config="$2"; stop="$3"; parent="$4"; ips="$5"
anchor="com.apple/250.KageVPN"
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

# sing-box's own stderr (Go panics, fatal errors) and exit status, so a silent death leaves a trace.
errlog="$(dirname "$config")/sing-box.stderr.log"
[ "$(wc -c < "$errlog" 2>/dev/null || echo 0)" -gt 1000000 ] && : > "$errlog"
"$bin" run -c "$config" >>"$errlog" 2>&1 &
child=$!

while kill -0 "$child" 2>/dev/null && [ ! -e "$stop" ] && kill -0 "$parent" 2>/dev/null; do
  sleep 0.5
done

if ! kill -0 "$child" 2>/dev/null; then
  wait "$child" 2>/dev/null
  status=$?
  echo "$(date '+%F %T') sing-box exited with status $status (stop requested: $([ -e "$stop" ] && echo yes || echo no))" >> "$errlog"
  if [ ! -e "$stop" ]; then
    {
      echo "$(date '+%F %T') context: exit status $status, supervisor pid $$, parent $parent"
      ps -axo pid,ppid,user,etime,comm 2>/dev/null | grep -iE 'vpn|tunnel|proxy|clash|v2ray|xray|wireguard|tailscale|openvpn|surge|tun2' | grep -v grep | head -6
    } >> "$errlog"
  fi
fi

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
