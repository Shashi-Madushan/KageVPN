# Domain entities
- **KnownNetwork** `{ id: String (lowercase gateway MAC, e.g. d8:d8:66:46:de:56), name: String, action: NetworkAction, lastSeen: u64 }`
- **NetworkAction** `connect | disconnect | ignore`
- **NetworkSettings** `{ enabled: bool, networks: Vec<KnownNetwork> }` in `Settings` (default: disabled, empty)
- **CurrentNetwork** `{ id, suggestedName }` read at runtime (not stored until seen)
- Runtime-only: `manual_override: bool` (set by a manual connect/disconnect, cleared on the next network change).
