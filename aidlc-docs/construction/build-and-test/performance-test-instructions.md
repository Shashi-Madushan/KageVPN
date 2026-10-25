# Performance
Not applicable: no hot paths were added. The network watcher adds two short external commands (`netstat`, `arp`) only when auto-connect is on and a change is detected. The diagnostics bundle reads at most the log files once per export.

## Round 4, pull request 1
Not applicable. Connecting now runs `sing-box version` once more to read the engine's build features (a few milliseconds); parsing is per imported link or file.

## Round 4, pull request 2
Not applicable. Setting or restoring runs about 10 `networksetup` calls per network service (well under a second for five services), off the async runtime, once per connect, stop or network change.
