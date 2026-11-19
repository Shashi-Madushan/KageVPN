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
#
# <dir> belongs to the user, so nothing in it is trusted. As root (or with KAGE_ISOLATE=1, which the
# tests use) the helper therefore keeps what it writes itself in a folder only root can change
# (/var/run/com.apzur.kage, or KAGE_RUN_DIR): sing-box's stderr log and a private copy of the
# configuration. The copy is checked before sing-box runs on it: it must log to <dir>/sing-box.log
# and may not name other files, a web UI to download, or a cache to write; those are options a
# program running as the user could otherwise use to make root write or read files anywhere.
# What root does write inside <dir> (the blocked marker, child.pid) is created without ever
# following a link someone put there.
bin="$1"; config="$2"; dir="$3"; parent="$4"
stop="$dir/stop"; down="$dir/down"; allowf="$dir/allow"; readyf="$dir/ready"; blockf="$dir/block"; holdmax="${KAGE_HOLD_MAX:-600}"; dnswindow="${KAGE_DNS_WINDOW:-20}"; blockedf="$dir/blocked"; startf="$dir/helper.start"; pidf="$dir/child.pid"; quitf="$dir/helper.quit"
anchor="com.apple/250.KageVPN"
if [ "$(id -u)" = 0 ] || [ -n "$KAGE_ISOLATE" ]; then
  isolate=yes
  rundir="${KAGE_RUN_DIR:-/var/run/com.apzur.kage}"
  mkdir -p "$rundir" 2>/dev/null
  # Only a real folder, not a link someone else put at that name.
  [ -d "$rundir" ] && [ ! -L "$rundir" ] || exit 1
  [ "$(id -u)" = 0 ] && chown root:wheel "$rundir" && chmod 755 "$rundir"
  active="$rundir/config.json"
else
  isolate=""
  rundir="$dir"
  active="$config"
fi
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
    | grep -E '^([0-9]{1,3}\.){3}[0-9]{1,3}$|^[0-9a-fA-F:]*:[0-9a-fA-F:]*:[0-9a-fA-F:.]*$' | sort -u | tr '\n' ' ' | sed 's/ *$//'
}

# Creates the marker the app watches. It is made only if nothing is there: a link left at that name
# is removed first and, if one appears again before the file is made, the write is refused instead of
# following it to whatever file it points at.
mark_blocked() {
  [ -L "$blockedf" ] && rm -f "$blockedf"
  ( set -C; : > "$blockedf" ) 2>/dev/null
  return 0
}

# Writes the pid file in one step, never through a link at either name.
write_pid() {
  rm -f "$pidf.tmp"
  ( set -C; echo "$1" > "$pidf.tmp" ) 2>/dev/null && mv -f "$pidf.tmp" "$pidf"
}

# Whether the configuration copy may be run by root: valid JSON that logs to <dir>/sing-box.log, and
# that names no other file to read or write, no web UI to download and no cache to keep. The app
# writes none of those, so refusing them costs nothing. `plutil` ships with macOS.
safe_config() {
  c="$1"
  [ "$(plutil -extract log.output raw -o - "$c" 2>/dev/null)" = "$dir/sing-box.log" ] || return 1
  plutil -convert xml1 -o - "$c" 2>/dev/null \
    | grep -Eq '<key>(external_ui|external_ui_download_url|external_ui_download_detour|cache_file|v2ray_api|certificate_path|certificate_directory_path|key_path|private_key_path|config_path|state_directory|plugin|plugin_opts|ntp)</key>' && return 1
  # Local rule-set files may only be the lists the app downloaded into its own folder.
  paths=$(plutil -extract route.rule_set json -o - "$c" 2>/dev/null | grep -o '"path" *: *"[^"]*"' | sed 's/^"path" *: *"//; s/"$//; s#\\/#/#g')
  [ -z "$paths" ] && return 0
  echo "$paths" | while IFS= read -r p; do
    case "$p" in "$dir"/rule-sets/*.srs) case "$p" in *..*) exit 1 ;; esac ;; *) exit 1 ;; esac
  done
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
  mark_blocked
}

release_block() {
  dns_until=0
  if [ -n "$token" ]; then
    pfctl -a "$anchor" -F all 2>/dev/null
    pfctl -X "$token" 2>/dev/null
    token=""
  fi
  # Last, so the marker never says "released" before the firewall really is.
  rm -f "$blockedf"
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
errlog="$rundir/sing-box.stderr.log"

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

  [ -L "$errlog" ] && rm -f "$errlog"
  [ "$(wc -c < "$errlog" 2>/dev/null || echo 0)" -gt 1000000 ] && : > "$errlog"
  # sing-box appends to this path as root; a link left there would send it to another file.
  [ -L "$dir/sing-box.log" ] && rm -f "$dir/sing-box.log"
  runconfig="$config"
  if [ -n "$isolate" ]; then
    # A copy only root can touch, so what was checked is what runs.
    # Written out again by plutil, so sing-box reads exactly what was checked (a file with a key
    # given twice could otherwise be read one way here and another way there).
    rm -f "$active" "$active.raw"
    if ! ( umask 077; cp "$config" "$active.raw" && plutil -convert json -o "$active" "$active.raw" ) 2>/dev/null || ! safe_config "$active"; then
      rm -f "$active" "$active.raw"
      echo "$(date '+%F %T') refused to start: the configuration is not one this app wrote" >> "$errlog"
      continue
    fi
    rm -f "$active.raw"
    runconfig="$active"
  fi
  "$bin" run -c "$runconfig" >>"$errlog" 2>&1 &
  child=$!
  write_pid "$child"

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
if [ -n "$isolate" ]; then rm -f "$active" "$active.raw"; fi
