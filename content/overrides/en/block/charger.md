source: src/main/resources/assets/oc2r/doc/en_us/block/charger.md
source_sha256: 0f3eda175a204c1047eb0a96d41c72645025df1a5649d6f4b6b2cea5f231eee5
# Charger
`oc2r:charger`

The charger transfers energy from a connected power source to energy consumers on top of it. It is also the **robot dock station**: a short-range radio access point and an Ethernet-to-radio bridge for the [ORA radio network](/en/reference/ora-radio).

## Charging

The wireless energy transfer supports any consumer accepting RF energy. Blocks and entities are charged, as are items in regular item containers (such as blocks and entities). This is particularly useful to recharge [robots](/en/item/robot) or energy consuming devices in your inventory.

## Robot dock

Every charger is a dock. A [robot](/en/item/robot) that is enrolled in a radio network attaches to the nearest dock within `robot_dock_range` (10 blocks by default, 1–64 allowed, circular in X/Z, boundary included) when no tower is in its own range. The dock must in turn have a [GPS tower](/en/block/gps_tower) of the same network within tower range. Radii are never added and the dock does not relay between towers. A dock is never registered anywhere and never loads chunks; a broken, replaced or unloaded dock stops counting immediately.

## Ethernet bridge

The charger accepts the blue network cable through an ordinary [network connector](/en/block/network_connector). A cable alone does not join a network: the dock has its own radio identity and the network administrator must invite it like any device. The `charger` device on the bus offers `getRadioEndpointId()`, `joinRadioNetwork(invite, secret)`, `getRadioAddress()` and `getRadioStatus()`; the invitation is issued on a tower with `issueRadioInvite(adminSecret, endpointId)`. Frames from the dock's connectors then enter the radio network through the nearest tower of the same network, and radio frames are written to every connector on the dock. An uninvited or revoked dock passes nothing.

See [ORA radio network](/en/reference/ora-radio) for enrolment, limits and the configuration keys.

<!-- wiki-recipes -->
## Crafting recipes

- [Charger](/en/recipes#recipe-charger)
<!-- /wiki-recipes -->
