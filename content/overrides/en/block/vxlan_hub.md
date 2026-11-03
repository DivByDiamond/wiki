source: src/main/java/li/cil/oc2/common/blockentity/network/vxlan/VxlanBlockEntity.java
source_sha256: 5a9c02cb0cfeed7f5545995ec06f7a5260c6532958d774a60872a26e5fdcb8e1
# VXLAN hub
`oc2r:vxlan_hub`

Bridges an in-game Ethernet network to an external VXLAN tunnel over UDP. Connect it to network devices rather than the device bus. External connectivity requires the server administrator to enable and configure the common config vxlan section. Local forwarding acts as a hub. It is separate from paired tunnel cards and GPS radio networks.

[Wiki](https://wiki.loki-code.dev/en/reference/networking).
