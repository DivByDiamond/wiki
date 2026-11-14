source: docs/RADIO-NETWORK.md
source_sha256: 4b2f0cc4edbbc1ec74a3fbe43e32104af6d13952696746a4fa81198167442af1
# ORA radio network

The radio network joins ordinary cable networks through [GPS towers](/en/block/gps_tower) and carries Ethernet frames, so TCP/UDP programs on a PC work unchanged. [Robots](/en/item/robot), [tablets](/en/item/tablet), the [robot dock](/en/block/charger) and the [wireless access point](/en/block/wireless_access_point) join it as radio clients.

```text
PC A → connector + blue cable → tower A ↔ tower B ↔ tower C → connector + blue cable → PC B
```

Radio enrolment is separate from GPS membership: knowing a GPS UUID or calling GPS `joinNetwork()` does not authorise radio frames. ORA does not assign IP addresses to guests; configure IPv4 in the guest separately.

## Coverage

Towers link up to 45 blocks apart horizontally (`dx² + dz² <= 45²`), ignoring height and walls. X=45, Z=0 is in range; X=32, Z=32 is not (about 45.25). Nothing crosses dimensions. Longer distances need a chain of enrolled towers. An unloaded tower still relays as its last known position, but the sending and receiving computers must be loaded, and the network never loads chunks.

## ORA addresses

**ORA = OC2R Radio Address**, written `7K3F:02Q9`: a four-character network prefix and a four-character endpoint number from the alphabet `0123456789ABCDEFGHJKMNPQRSTVWXYZ` (case-insensitive on input, `0000` reserved). The server picks free values at random, saves them with the world and never reuses a revoked one. An address is not a password, is not derived from coordinates and does not leave its dimension.

## Enrolment

First setup is done through the tower's device bus, which counts as trusted physical access. The `gps_tower` device exposes:

| Method | Purpose |
|---|---|
| `createRadioNetwork()` | create a network, returns `network` and `adminSecret` |
| `getRadioEndpointId()` | endpoint UUID to invite |
| `issueRadioInvite(adminSecret, endpointId)` | single-use invitation, returns `invite` and `secret` |
| `joinRadioNetwork(invite, secret)` | enrol this endpoint |
| `getRadioAddress()` / `getRadioStatus()` | address and enrolment state |
| `revokeRadioEndpoint(adminSecret, endpointId)` | revoke one endpoint |

```lua
local tower = assert(require("devices"):find("gps_tower"))
-- first PC, once; keep adminSecret
local network = tower:createRadioNetwork()
local inv = tower:issueRadioInvite(network.adminSecret, "SECOND-TOWER-ENDPOINT-UUID")
-- second PC
tower:joinRadioNetwork(inv.invite, inv.secret)
print(tower:getRadioAddress())
```

`find()` does not pick the nearest tower; use `devices:get(deviceId)` when several are attached. Every relay tower must be enrolled too.

### Approval-gated enrolment

`issueRadioInviteForApproval(adminSecret, endpointId, permissions, targetEndpointId, service)` makes an invitation that cannot admit by itself. The joining device calls `requestRadioEnrollment(invite, secret)`, which creates a pending request with no membership and no permission. An administrator then decides:

```lua
local pending = tower:listPendingEnrollments(adminSecret)
tower:approveEnrollment(adminSecret, pending[1].request, {"MESSAGE_SEND"})
-- or tower:denyEnrollment(adminSecret, pending[1].request)
-- on the joining device afterwards: tower:completeRadioEnrollment()
```

The caller must be an enrolled tower with `NETWORK_ADMIN` and know the admin secret. The device gets exactly the approved scopes. At most 16 requests are pending per network, one per device, and each expires after 6000 ticks (five minutes). The ORA address is assigned on approval.

## Robots, docks, access points and tablets

- A robot attaches directly to a tower within tower range (`gps_tower_range`, 45). Otherwise it attaches to the nearest [robot dock](/en/block/charger) or [wireless access point](/en/block/wireless_access_point) within 10 blocks (`robot_dock_range`) that has a tower of the same network within tower range. Radii are never added and docks do not relay between towers.
- Docks and access points never load chunks; a broken or unloaded one stops counting at once.
- Through a dock a robot gets the same control as on a tower: telemetry, commands, ping, device list, permission checks. Leaving the radius is signal loss; a command sent then ends as `DISCONNECTED` or `TIMED_OUT` and is not replayed.
- A robot command result is authoritative: the guest binds a queued `move`/`turn` action with `bindRadioCommand(requestId, actionId)` and the server reports `COMPLETED` or `FAILED` when it really finishes.
- A tablet's identity lives in the item: one active holder at a time, a new secret on every activation, so copies never become a second identity. Radio drops when nobody holds the item.

### Dock Ethernet

A dock accepts the blue cable through a network connector, but the cable alone joins nothing: the dock has its own endpoint and must be invited. The `charger` device has `getRadioEndpointId`, `joinRadioNetwork`, `getRadioAddress` and `getRadioStatus`. Frames from the dock's connectors go into the radio through the nearest tower of the same network, and radio frames are written to all its connectors.

## Delay, ping and limits

- Each hop takes one tick but total propagation is capped at `radioMaxDeliveryTicks` (2 ticks, 100 ms at 20 TPS). Each sender is limited to `radioLinkBytesPerTick` (1024 B/tick, about 20 KB/s).
- `pingRadio(address, requestId)` measures the real round trip; the answer arrives in `pollRadioEvents()` as `roundTripNanos`. `route.hops` and `route.nominalMillis` are schedule estimates.
- The hop TTL belongs to the OC2R network bus and is separate from the IP TTL. Old packets expire and queues are not saved on restart.
- **TLS is not implemented.** `getRadioStatus()` reports `trusted-local-plaintext-no-tls`.
- Telemetry fields report `ageTicks`, `ageMillis` and a state of `ONLINE`, `STALE` or `OFFLINE` (`getRadioTelemetryStatus(subscription)`).

Settings, in the `[gameplay]` section of the common config (restart after changing the radio limits):

```toml
gps_tower_range = 45
robot_dock_range = 10
radioHopDelayTicks = 1
radioMaxDeliveryTicks = 2
radioLinkBytesPerTick = 1024
radioMaxHops = 64
radioMaxFrameBytes = 2048
radioQueueLimit = 4096
radioNetworkQueueLimit = 1024
radioDeviceQueueLimit = 256
radioPacketLifetimeTicks = 200
```

Secrets are 256 random bits and the registry stores only SHA-256 hashes. Revocation affects one device; re-inviting gives a new generation, address and secret, and an older secret never authenticates again.
