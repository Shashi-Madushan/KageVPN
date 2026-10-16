# Performance
Not applicable: no hot paths were added. The network watcher adds two short external commands (`netstat`, `arp`) only when auto-connect is on and a change is detected. The diagnostics bundle reads at most the log files once per export.
