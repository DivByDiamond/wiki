source: src/main/java/li/cil/oc2/common/blockentity/misc/PciCardCageBlockEntity.java
source_sha256: 8edbedb0b13a31d221d3c33a347036e0f6b53e3d3a89118831270c4af479bbe5
# Корпус PCI
`oc2r:pci_card_cage`

Предоставляет гостевой машине нативный корневой порт PCI с окном памяти 16 МиБ. Подключите заднюю сторону к интерфейсной шине и подайте питание, если оно включено в конфиге. Это нативное устройство для драйвера гостевой ОС. Оно не выдаёт дополнительные предметные слоты для карт и не имеет высокоуровневого Lua API.

[Wiki](https://wiki.loki-code.dev/en/reference/devices).
