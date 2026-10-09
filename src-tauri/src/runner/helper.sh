#!/bin/sh
# usage: helper.sh <sing-box> <config> <dir> <parent-pid>
# Started once per app launch through a single administrator prompt, then kept running as root
# until the app exits. With parent "-" it is the installed launchd service instead: it never exits
# on its own, and takes the app's pid from line 2 of helper.start. Each connect writes <dir>/helper.start (first line: kill-switch IPs, "-"
# for none listed, or "none" to leave the kill switch off); the helper starts sing-box, writes
# its pid to <dir>/child.pid, and stops it when <dir>/stop appears. Disconnecting and
# reconnecting therefore never prompt again. <dir>/helper.quit makes the helper exit.
bin="$1"; config="$2"; dir="$3"; parent="$4"
stop="$dir/stop"; startf="$dir/helper.start"; pidf="$dir/child.pid"; quitf="$dir/helper.quit"
anchor="com.apple/250.KageVPN"
token=""
child=""

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
  [ -z "$token" ] && return
  pfctl -a "$anchor" -F all 2>/dev/null
  pfctl -X "$token" 2>/dev/null
  token=""
}

# "-" stands for "no parent": the launchd root service, which has no app to watch of its own.
alive() { [ "$1" = "-" ] || kill -0 "$1" 2>/dev/null; }

# Status 128+N means sing-box was killed by signal N (143 = TERM, 129 = HUP, 130 = INT); 0 is its
# own clean exit. The process list names other VPN or proxy tools that could have interfered.
note_context() {
  {
    echo "$(date '+%F %T') context: exit status $status, helper pid $$, parent $parent"
    ps -axo pid,ppid,user,etime,comm 2>/dev/null | grep -iE 'vpn|tunnel|proxy|clash|v2ray|xray|wireguard|tailscale|openvpn|surge|tun2' | grep -v grep | head -6
  } >> "$errlog"
}

# A signal sent to the helper itself is logged, so the cause is not lost.
trap 'echo "$(date "+%F %T") helper received a signal" >> "$errlog"; [ -n "$child" ] && kill "$child" 2>/dev/null; exit 0' TERM INT HUP

# sing-box's own stderr (Go panics, fatal errors) and exit status, so a silent death leaves a trace.
errlog="$dir/sing-box.stderr.log"

while alive "$parent" && [ ! -e "$quitf" ]; do
  if [ ! -e "$startf" ]; then
    sleep 0.5
    continue
  fi

  ips=$(head -n 1 "$startf")
  # The root service has no parent; each run names the app that asked for it on line 2, and
  # sing-box stops if that app disappears.
  owner="$parent"
  [ "$parent" = "-" ] && owner=$(sed -n 2p "$startf")
  [ -z "$owner" ] && owner="-"
  rm -f "$startf" "$stop"
  release_block

  [ "$(wc -c < "$errlog" 2>/dev/null || echo 0)" -gt 1000000 ] && : > "$errlog"
  "$bin" run -c "$config" >>"$errlog" 2>&1 &
  child=$!
  echo "$child" > "$pidf.tmp" && mv "$pidf.tmp" "$pidf"

  while alive "$child" && [ ! -e "$stop" ] && [ ! -e "$quitf" ] && alive "$owner"; do
    sleep 0.5
  done

  if ! alive "$child"; then
    wait "$child" 2>/dev/null
    status=$?
    echo "$(date '+%F %T') sing-box exited with status $status (stop requested: $([ -e "$stop" ] && echo yes || echo no))" >> "$errlog"
    [ ! -e "$stop" ] && note_context
  fi

  # sing-box died on its own: with a kill switch, block traffic until the app asks for a new run.
  if [ "$ips" != "none" ] && [ ! -e "$stop" ] && alive "$owner" && ! alive "$child"; then
    engage_block
    while [ ! -e "$stop" ] && [ ! -e "$startf" ] && [ ! -e "$quitf" ] && alive "$owner"; do
      sleep 0.5
    done
    release_block
  fi

  kill "$child" 2>/dev/null
  wait "$child" 2>/dev/null
  rm -f "$pidf" "$stop"
  child=""
done

release_block
rm -f "$quitf" "$pidf"
