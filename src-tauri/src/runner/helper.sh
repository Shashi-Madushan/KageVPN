#!/bin/sh
# usage: helper.sh <sing-box> <config> <dir> <parent-pid>
# Started once per app launch through a single administrator prompt, then kept running as root
# until the app exits. With parent "-" it is the installed launchd service instead: it never exits
# on its own, and takes the app's pid from line 2 of helper.start. Each connect writes <dir>/helper.start (first line: kill-switch IPs, "-"
# for none listed, or "none" to leave the kill switch off); the helper starts sing-box, writes
# its pid to <dir>/child.pid, and stops it when <dir>/stop appears. Disconnecting and
# reconnecting therefore never prompt again. <dir>/helper.quit makes the helper exit.
# <dir>/stop ends the run and releases any kill-switch block. <dir>/down also ends the run but,
# when the kill switch is on for it, keeps (or puts up) the block until the next run is asked for,
# <dir>/stop appears, or the app is gone: a failed reconnect must not leave traffic exposed.
# A block that is up when a new run starts stays up while sing-box comes up, until the app writes
# <dir>/ready (the tunnel was verified) or <dir>/stop; <dir>/block puts the block up while sing-box
# is alive but the tunnel is not carrying traffic, and <dir>/ready takes it down again. A block the
# app never releases is dropped after KAGE_HOLD_MAX seconds (default 600) so a wedged app cannot
# keep the Mac offline.
# While a new run comes up behind a block, outgoing DNS (port 53) is let through for KAGE_DNS_WINDOW
# seconds (default 20) so sing-box can resolve its server's name, then the rules are reloaded
# without it. <dir>/blocked exists exactly while a block is up, so the app can tell the user.
bin="$1"; config="$2"; dir="$3"; parent="$4"
stop="$dir/stop"; down="$dir/down"; allowf="$dir/allow"; readyf="$dir/ready"; blockf="$dir/block"; holdmax="${KAGE_HOLD_MAX:-600}"; dnswindow="${KAGE_DNS_WINDOW:-20}"; blockedf="$dir/blocked"; startf="$dir/helper.start"; pidf="$dir/child.pid"; quitf="$dir/helper.quit"
anchor="com.apple/250.KageVPN"
token=""
child=""
held_since=0
dns_until=0

# The server addresses the block lets through: the ones named for this run plus the app's
# refreshed list in <dir>/allow (a name can resolve to other addresses later). Both files are
# written by the user-level app, so only things shaped like an IPv4 or IPv6 address get through:
# one bad word would make pfctl reject the whole rule set and leave traffic unblocked.
allowed_addresses() {
  { echo "$ips"; head -n 1 "$allowf" 2>/dev/null; } | tr ',' '\n' | tr -d ' ' \
    | grep -E '^[0-9a-fA-F:.]*[.:][0-9a-fA-F:.]*$' | sort -u | tr '\n' ' ' | sed 's/ *$//'
}

# Loads the rule set (again, when the block is already up, so a refreshed address list applies)
# and turns the firewall on once.
engage_block() {
  list=$(allowed_addresses)
  {
    echo "pass quick on lo0 all"
    echo "pass out quick to { 10.0.0.0/8 172.16.0.0/12 192.168.0.0/16 169.254.0.0/16 224.0.0.0/4 255.255.255.255 }"
    # IPv6 needs neighbour and router discovery and link-local traffic to reach its own router at all.
    echo "pass out quick inet6 proto icmp6 all icmp6-type { routersol, routeradv, neighbrsol, neighbradv }"
    echo "pass out quick inet6 from fe80::/10 to fe80::/10"
    [ -n "$list" ] && echo "pass out quick to { $list }"
    [ "$dns_until" -gt "$(date +%s)" ] && echo "pass out quick proto { udp tcp } to any port 53"
    echo "block drop out all"
  } | pfctl -a "$anchor" -f - 2>/dev/null
  if [ -z "$token" ]; then
    token=$(pfctl -E 2>&1 | sed -n 's/.*Token : //p')
    held_since=$(date +%s)
  fi
  : > "$blockedf"
}

release_block() {
  dns_until=0
  rm -f "$blockedf"
  [ -z "$token" ] && return
  pfctl -a "$anchor" -F all 2>/dev/null
  pfctl -X "$token" 2>/dev/null
  token=""
}

# Runs every half second while sing-box is running: the app's signals about the block.
tick() {
  if [ -e "$blockf" ]; then
    rm -f "$blockf"
    [ "$ips" != "none" ] && engage_block
  fi
  if [ -e "$readyf" ]; then
    rm -f "$readyf"
    release_block
  fi
  if [ "$dns_until" -ne 0 ] && [ "$(date +%s)" -ge "$dns_until" ]; then
    dns_until=0
    [ -n "$token" ] && engage_block
  fi
  if [ -n "$token" ] && [ $(( $(date +%s) - held_since )) -ge "$holdmax" ]; then
    echo "$(date '+%F %T') the app never released the kill-switch block: releasing it" >> "$errlog"
    release_block
  fi
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

rm -f "$blockedf"

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
  rm -f "$startf" "$stop" "$down" "$readyf" "$blockf"
  # A block still up from the run before stays until the new run is verified (see "ready"), with
  # the address list refreshed. Without a kill switch for this run it goes now.
  if [ "$ips" = "none" ]; then release_block; elif [ -n "$token" ]; then
    dns_until=$(( $(date +%s) + dnswindow ))
    engage_block
  fi

  [ "$(wc -c < "$errlog" 2>/dev/null || echo 0)" -gt 1000000 ] && : > "$errlog"
  "$bin" run -c "$config" >>"$errlog" 2>&1 &
  child=$!
  echo "$child" > "$pidf.tmp" && mv "$pidf.tmp" "$pidf"

  while alive "$child" && [ ! -e "$stop" ] && [ ! -e "$down" ] && [ ! -e "$quitf" ] && alive "$owner"; do
    tick
    sleep 0.5
  done

  # "down": end the run now, so the block below is put up exactly as if sing-box had died.
  held=""
  if [ -e "$down" ]; then
    held=yes
    echo "$(date '+%F %T') down requested: stopping sing-box and holding the kill switch" >> "$errlog"
    kill "$child" 2>/dev/null
    wait "$child" 2>/dev/null
  fi

  if [ -z "$held" ] && ! alive "$child"; then
    wait "$child" 2>/dev/null
    status=$?
    echo "$(date '+%F %T') sing-box exited with status $status (stop requested: $([ -e "$stop" ] && echo yes || echo no))" >> "$errlog"
    [ ! -e "$stop" ] && note_context
  fi

  # sing-box died on its own, or "down" was asked: with a kill switch, block traffic until the app
  # asks for a new run. "stop" always wins and releases.
  if [ "$ips" != "none" ] && [ ! -e "$stop" ] && [ ! -e "$quitf" ] && alive "$owner" && ! alive "$child"; then
    engage_block
    while [ ! -e "$stop" ] && [ ! -e "$startf" ] && [ ! -e "$quitf" ] && alive "$owner"; do
      sleep 0.5
    done
    # A new run keeps the block until it is verified; anything else ends it.
    [ -e "$startf" ] && [ ! -e "$stop" ] && [ ! -e "$quitf" ] || release_block
  fi

  # Stopped, quit or abandoned: whatever block is still up goes with it.
  if [ -e "$stop" ] || [ -e "$quitf" ] || ! alive "$owner"; then release_block; fi

  kill "$child" 2>/dev/null
  wait "$child" 2>/dev/null
  rm -f "$pidf" "$stop" "$down" "$readyf" "$blockf"
  child=""
done

release_block
rm -f "$quitf" "$pidf"
