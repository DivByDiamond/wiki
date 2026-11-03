source: src/main/java/li/cil/oc2/common/blockentity/network/vxlan/VxlanBlockEntity.java
source_sha256: 5a9c02cb0cfeed7f5545995ec06f7a5260c6532958d774a60872a26e5fdcb8e1
# VXLAN-концентратор
`oc2r:vxlan_hub`

Соединяет игровую Ethernet-сеть с внешним VXLAN-туннелем по UDP. Подключается к сетевым устройствам, а не к интерфейсной шине. Для внешнего соединения администратор должен включить и настроить раздел `vxlan` общего конфига. Обычный локальный обмен работает как у концентратора. Это отдельный механизм от туннельных карт и GPS-радиосети.

[Wiki](https://wiki.loki-code.dev/en/reference/networking).
