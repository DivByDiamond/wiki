source: src/main/java/li/cil/oc2/common/blockentity/misc/PciCardCageBlockEntity.java
source_sha256: 8edbedb0b13a31d221d3c33a347036e0f6b53e3d3a89118831270c4af479bbe5
# PCI card cage
`oc2r:pci_card_cage`

Exposes a native PCI root port with a 16 MiB memory window to the guest. Connect its rear side to the device bus and supply power when required by server settings. It is a native device for guest drivers, not extra item slots for expansion cards. It has no high-level Lua API.

[Wiki](https://wiki.loki-code.dev/en/reference/devices).
