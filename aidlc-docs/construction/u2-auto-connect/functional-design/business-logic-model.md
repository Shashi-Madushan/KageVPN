# Business logic model
```mermaid
flowchart TD
    A["Trigger: start, wake, network change"] --> B["Settle delay"]
    B --> C{"Enabled and not stood down?"}
    C -->|no| Z["Stop"]
    C -->|yes| D["Read gateway MAC"]
    D --> E{"MAC found?"}
    E -->|no| Z
    E -->|yes| F{"Known network?"}
    F -->|no| G["Store as ignore and ask via notification"]
    F -->|yes| H{"Action"}
    H -->|connect| I["Connect if disconnected"]
    H -->|disconnect| J["Disconnect and release kill switch"]
    H -->|ignore| Z
```
Text alternative: after a trigger and a settle delay, if the feature is on and not stood down, read the gateway MAC; an unknown network is stored as ignore and the user is asked; a known one runs its action (connect, disconnect, ignore).

Pure core for tests: `decide(settings, current_id, state, manual_override) -> Decision { Nothing | AskAbout(id) | Connect | Disconnect }`. Gateway reading and notifications are thin wrappers around it.
