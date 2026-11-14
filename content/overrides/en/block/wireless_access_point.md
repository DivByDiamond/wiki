source: docs/PERIPHERALS.md
source_sha256: 2d7d3634607678e40a830bb76caef2b5e8f93d4252893c6422205116f08766cc
# Wireless access point
`oc2r:wireless_access_point`

A short-range radio access point for [robots](/en/item/robot), [tablets](/en/item/tablet) and computers. Like the [robot dock](/en/block/charger), it gives clients inside its radius a way onto the [ORA radio network](/en/reference/ora-radio) of the tower it is enrolled in.

## Range

Clients inside `robot_dock_range` (10 blocks by default) can reach a tower of the access point's network. A loaded tower must be within tower range of the point (`gps_tower_range`, 45 blocks by default). Ranges are circular in X/Z and ignore height. The point does not relay between towers and radii are never added. Unloading or breaking it removes its coverage; no chunk is force-loaded.

## Connecting it

- **Device bus**: place a device-bus cable or [bus interface](/en/block/bus_interface) below the block (an interface on an adjacent side also works). A running computer on that bus gets radio messages, telemetry and control RPC.
- **Ethernet**: put a [network connector](/en/block/network_connector) against the underside and use the blue network cable. The network-interface port exists only on the bottom face.

The off texture is shown while the point is disconnected. Once it is attached to an authenticated tower, the three "on" frames play at six ticks per frame.

## Enrolment

The access point is its own radio identity and has to be invited like any device:

1. Read `getRadioEndpointId()` through the device bus.
2. From the network administrator's tower RPC, issue an invitation for that endpoint and approve it.
3. Call `joinRadioNetwork(invite, secret)` on the access point.

`getRadioAddress()` and `getRadioStatus()` show its address and connection state without the credential. `getRadioStatus().guestOnline` tells the guest service apart from infrastructure connectivity. Permissions belong to the access point's enrolled identity, not to the tower. An unenrolled, revoked or duplicate identity can neither bridge traffic nor provide coverage.

A running computer on the bus can register services, send and receive ORA messages, ping, discover devices, publish and subscribe telemetry and send or answer commands. Named services survive save and load, but an offline guest cannot receive commands. When the computer stops or the bus is detached, guest services disconnect and pending work is cancelled while client coverage and the Ethernet bridge stay up. The `gps_tower` alias in discovery only keeps `ochat` working; it adds no positioning.
