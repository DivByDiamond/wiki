source: src/main/java/li/cil/oc2/common/blockentity/network/vxlan/VxlanBlockEntity.java
source_sha256: d3b1001b660d1acab4d2fb02c34e0dcd2cb6d9ef7ec2a258437bf1e302762cbe
# VXLAN hub
`oc2r:vxlan_hub`

Bridges an in-game Ethernet network to an external VXLAN tunnel over UDP. Connect it to network devices rather than the device bus. External connectivity requires the server administrator to enable and configure the common config vxlan section (it is off by default). Each hub has a random VNI; right-click the hub with an empty hand to see it together with the `ip link` command for the other end (the server also logs it when the hub registers), and use the same number there. The hub accepts datagrams from the configured remote host on any source port. Local forwarding acts as a hub. It is separate from paired tunnel cards, GPS radio networks and the internet card, which does not need VXLAN.

[Wiki](https://wiki.loki-code.dev/en/reference/networking).
