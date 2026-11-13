source: src/main/java/li/cil/oc2/common/blockentity/network/vxlan/VxlanBlockEntity.java
source_sha256: 73ea41302e96da23ce35e22c105ca1c792b37646ec76fbd3c4e35f6b7cd0c519
# VXLAN hub
`oc2r:vxlan_hub`

Bridges an in-game Ethernet network to an external VXLAN tunnel over UDP. Connect it to network devices rather than the device bus. External connectivity requires the server administrator to enable and configure the common config vxlan section (it is off by default). Each hub has a random VNI that the server writes to its log when the hub registers; use the same number at the other end of the tunnel. Local forwarding acts as a hub. It is separate from paired tunnel cards, GPS radio networks and the internet card, which does not need VXLAN.

[Wiki](https://wiki.loki-code.dev/en/reference/networking).
