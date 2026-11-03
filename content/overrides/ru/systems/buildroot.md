source: docs/BUILDROOT.md
source_sha256: d21744b90362ac061fb7e042c7937b9c277a5d63f3f965ad26b8b6de7497be0b
# Buildroot и образ Linux

Комплектный образ Linux собирается проектом `fnuecke/sedna-buildroot`. Зависимость загружается из [Maven-репозитория](https://fnuecke.github.io/maven) скриптом `scripts/download-libs.sh` в каталог `libs/`.

Образ содержит 64-битное ядро RISC-V и корневую файловую систему. Они упакованы в `sedna-buildroot-<version>.jar`, который входит в JAR мода как вложенная зависимость.

| Файл в каталоге `generated/` внутри JAR | Назначение |
|---|---|
| `firmware.bin` | Минимальное загрузочное ПЗУ RISC-V; записывается во flash-память для `MinuxFirmware` |
| `firmware.properties` | Размер области прошивки `regionSize`, равный 8192 |
| `rootfs.ext2` | Корневая файловая система с ядром. Подключается как блочное устройство VirtIO RFS через `BuiltinDevices` и `BuildrootBlockDeviceData` |

Начиная с линии 0.2.x ядро не хранится внутри прошивки. Загрузочное ПЗУ читает его с диска: flash-память содержит `firmware.bin`, а `/boot/Image` должен быть доступен на блочном устройстве VirtIO. Отдельная загрузочная файловая система `bootfs.squashfs` и устройство BFS больше не создаются для новых виртуальных машин.

Этот раздел относится к Linux. У OnyxOS свой корневой образ и собственные программы; наличие комплектного Linux-образа не означает, что его утилиты можно выполнять в OnyxOS.

## 9p и каталог /mnt/builtin

В ядре линии 0.2.x включена файловая система 9p: `CONFIG_9P_FS=y`. Это проверялось по строкам `v9fs` в `/boot/Image`.

Скрипт `/etc/init.d/S90virtio9p` монтирует каждую метку virtio-9p в `/mnt/<tag>`. Поэтому общий каталог мода `builtin`, содержащий скрипты, `librpc` и документацию, доступен в Linux по пути `/mnt/builtin`.

## Tiny C Compiler

Начиная с `sedna-buildroot 0.0.64` в образ включён TinyCC. В настройках Buildroot используется:

```text
BR2_PACKAGE_TINYCC=y
```

В комплекте `sedna-buildroot-0.0.64.jar` при проверке были найдены:

```text
usr/bin/tcc
usr/lib/tcc/include/*.h
usr/lib/tcc/runmain.o
licenses/tinycc-*/COPYING
```

Размер исполняемого файла `tcc` составлял около 297 КБ. В каталоге заголовков есть `stddef.h`, `stdarg.h`, `tcclib.h` и другие файлы.

Скомпилировать пример C внутри гостевой ОС можно так, находясь в каталоге с исходниками:

```sh
tcc redstone_blink.c rpc.c -o redstone_blink
./redstone_blink up
```

Комплектная RPC-библиотека лежит в `/mnt/builtin/lib/rpc/`. Она общается с устройствами Minecraft через `/dev/hvc0`. Среди примеров есть `redstone_blink.c`, `note_block_player.c` и `example_raii.cpp`.

## Где задаётся версия

- В `gradle.properties`: `sedna_buildroot_version=0.2.10`.
- В `build.gradle.kts`: зависимость `implementation("li.cil.sedna:sedna-buildroot:${sedna_buildroot_version}")`.
- В `scripts/download-libs.sh`: загрузка JAR из `https://fnuecke.github.io/maven` в `libs/`.

## История образа 0.0.72-oc2r1

В Minux 0.0.71 из гостевого ядра убрали 9p, что сломало монтирование `/mnt/builtin`. Проблема описана в [minux#12](https://github.com/North-Western-Development/minux/issues/12) и OC2R #17.

До перехода на 0.2.x OC2R поставлял собственный образ: файлы Minux 0.0.71 и Linux 6.12.104 с `CONFIG_9P_FS=y`. Он распространялся через релиз `bundled-deps` под версией `0.0.72-oc2r1`.

В линии 0.2.x поддержка 9p снова встроена в ядро, поэтому собственный промежуточный образ больше не используется.

## Пересборка образа

Образ собирается вне этого репозитория. Нужен полный инструментарий кросс-компиляции Buildroot либо CI исходного проекта.

После сборки обновите `sedna_buildroot_version` в `gradle.properties`, выполните `scripts/download-libs.sh --force` и пересоберите мод. Обновление зависимости само по себе не переносит файлы в уже существующие частные корневые диски виртуальных машин.
