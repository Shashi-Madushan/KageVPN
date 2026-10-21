# U2 domain entities

## AppCap (setting; many per user, one per app)
| Field | Meaning |
|---|---|
| `path` | The app's bundle path, for example `/Applications/Slack.app`. Identity of the cap. |
| `limitBytes` | 1 MB to 10 TB. |
| `period` | `day`, `week` or `month`. |
| `atLimit` | `notify` (alerts only) or `offerBlock` (alerts, and a Block now choice at 100 percent). |

Stored with the usage settings (new field, defaults to none, old files load). An app with no cap has no entry.

## CapTally (running count; one per capped app)
| Field | Meaning |
|---|---|
| `path` | Same key as the cap. |
| `periodStart` | Unix seconds of the start of the period being counted. |
| `up`, `down` | Bytes of this app's connections through the VPN since `periodStart`. |
| `alertLevel` | 0, 80 or 100: the highest alert already sent this period. |
| `offer` | `none`, `pending` (100 percent reached, Block now offered), `taken` (blocked by the cap), `dismissed` (user chose not to block this period). |

Stored in a new table `app_cap_tally` in the usage database, separate from the usage history, so Clear details and the retention setting never touch it. "Clear all usage" does not touch it either (it is a billing count, not history); removing the cap deletes its tally.

## Cap rule (a routing rule)
A `Rule` with `origin = cap`, `kind = app`, `value = path`, `action = block`, `enabled`. Created by Block now, removed by period rollover, Unblock or removing the cap. It sits at the top of the rule list so it wins over the user's own rule for the same app, which is kept underneath.

## CapStatus (read model sent to the UI)
`path`, `name`, `limitBytes`, `period`, `atLimit`, `used` (up plus down), `up`, `down`, `percent`, `level`, `state` (`ok`, `warn80`, `over`, `blocked`, `dismissed`, `notCounting`), `periodStart`, `periodEnd`.
