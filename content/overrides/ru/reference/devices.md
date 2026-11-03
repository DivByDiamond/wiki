source: docs/DEVICES.md
source_sha256: 3e0f289397375c396781cae5715ad3c72cf250bee86b3dcdc836636cadb1c1e0
# Система устройств

Устройства OC2R подключаются к виртуальному компьютеру через интерфейсную шину. Для нативного оборудования и высокоуровневых вызовов используются разные API. Этот справочник описывает их со стороны разработчика; примеры для гостевых программ есть в разделе [скриптов](/ru/scripting).

## VMDevice: нативные устройства

`VMDevice` регистрирует области ввода-вывода в памяти RISC-V, или MMIO, и линии прерываний. Для работы с таким оборудованием гостевой ОС нужен драйвер.

Жизненный цикл:

1. `mount(VMContext)` выделяет ресурсы, подключает MMIO и прерывания. Метод возвращает `VMDeviceLoadResult`.
2. `unmount()` приостанавливает ввод-вывод и освобождает подключения к работающей машине.
3. `dispose()` окончательно освобождает ресурсы удалённого устройства. Метод унаследован от `Device`.

При выгрузке чанка устройство приостанавливается, а после загрузки может быть смонтировано снова. Если при запуске другое устройство не удалось смонтировать, уже подключённые устройства также могут пройти остановку и освобождение ресурсов.

| Реализация | Назначение |
|---|---|
| `DiskDriveDevice` | Чтение и запись дискет; в текущем оборудовании используется медиастанцией |
| `HardDriveDevice` | Блочное хранилище |
| `KeyboardDevice` | Ввод с клавиатуры |
| `MonitorDevice` | Экран с кадровым буфером |
| `NetworkInterfaceCardDevice` | Ethernet |
| `InternetCardDevice` | Подключение к внешней сети |
| `NetworkTunnelDevice` | Сетевой туннель |
| `MemoryDevice` | Оперативная память |
| `PciCardCageDevice` | Нативное устройство корпуса PCI |
| `ProjectorDevice` | Проектор |
| `FirmwareFlashStorageDevice` | Хранилище прошивки |
| `SimpleFramebufferDevice` | Кадровый буфер |
| `PciRootPortDevice` | Корневой порт PCI |

## RPCDevice: высокоуровневые вызовы

Высокоуровневые устройства принимают JSON-сообщения через последовательный порт VirtIO. В комплектном Linux канал доступен как `/dev/hvc0`. Специальный драйвер конкретного устройства не нужен: программа отправляет вызов метода и получает ответ.

Основная часть интерфейса:

```java
public interface RPCDevice extends Device {
    List<String> getTypeNames();
    List<RPCMethodGroup> getMethodGroups();
}
```

`getTypeNames()` возвращает имена типов, по которым гостевая программа может найти устройство. `getMethodGroups()` описывает доступные методы. В интерфейсе также есть стандартные методы жизненного цикла `mount()` и `unmount()`.

## ObjectDevice и аннотация @Callback

`ObjectDevice` оборачивает обычный Java-объект. Методы с `@Callback` становятся доступны через RPC. Наследоваться от `ObjectDevice` не нужно: это `final`-класс.

Пример объекта устройства:

```java
import java.util.Collection;
import java.util.List;
import li.cil.oc2.api.bus.device.object.Callback;
import li.cil.oc2.api.bus.device.object.NamedDevice;

public final class Thermometer implements NamedDevice {
    @Callback
    public double getTemperature() {
        return 36.6;
    }

    @Override
    public Collection<String> getDeviceTypeNames() {
        return List.of("thermometer");
    }
}
```

Такой объект передают в `new ObjectDevice(new Thermometer())`, затем предоставляют полученное устройство шине через провайдер. Само создание обёртки не регистрирует устройство в игровом мире.

| Аннотация | Что делает |
|---|---|
| `@Callback` | Открывает метод гостевой программе. Параметры `name` и `description` задают имя и описание; `synchronize` управляет выполнением на основном потоке |
| `@Parameter` | Задаёт имя и описание параметра |

По умолчанию `@Callback` выполняется синхронно на основном потоке сервера. Это нужно для обращения к миру и другим объектам, которые не поддерживают одновременный доступ из нескольких потоков. `synchronize = false` подходит только для методов, безопасность которых проверена отдельно.

Дополнительные интерфейсы объекта:

| Интерфейс | Назначение |
|---|---|
| `NamedDevice` | Дополнительные имена типов через `getDeviceTypeNames()` |
| `DocumentedDevice` | Документация методов и параметров через посетителя документации |
| `LifecycleAwareDevice` | Обработчики подключения, отключения и удаления устройства |

## Прошивка

`FirmwareLoader` является маркерным интерфейсом, который наследуется от `VMDevice`. По нему система проверяет, есть ли среди устройств загрузчик прошивки. Он не объявляет собственный метод `getFirmware()`.

Данные прошивок и их загрузка реализованы отдельными классами. Например, `MinuxFirmware` представляет комплектную прошивку Minux, а `ByteBufferFlashStorageDevice` работает с содержимым flash-памяти в буфере байтов.

## Провайдеры блоков и предметов

Провайдер связывает игровой объект с устройством виртуальной машины. Его основные методы:

```java
public interface BlockDeviceProvider {
    Optional<Device> getDevice(BlockDeviceQuery query);
}

public interface ItemDeviceProvider {
    Optional<ItemDevice> getDevice(ItemDeviceQuery query);
}
```

Это фрагменты интерфейсов. Если подходящего устройства нет, возвращается пустой `Optional`. Полные интерфейсы содержат и другие методы, например очистку состояния и расчёт энергопотребления предметного устройства.

Регистрация и поиск провайдеров находятся в `Providers.java`, `ProviderRegistry` и коде соответствующего загрузчика. Среди предметных провайдеров есть `CPUItemDeviceProvider`, `MemoryItemDeviceProvider`, `HardDriveItemDeviceProvider`, `NetworkInterfaceCardItemDeviceProvider` и `RedstoneInterfaceCardItemDeviceProvider`.

## Типы устройств и теги

| Тип | Тег предметов | Пример |
|---|---|---|
| `MEMORY` | `oc2r:devices/memory` | Модуль ОЗУ |
| `HARD_DRIVE` | `oc2r:devices/hard_drive` | Жёсткий диск |
| `FLASH_MEMORY` | `oc2r:devices/flash_memory` | Чип прошивки |
| `CARD` | `oc2r:devices/card` | Карта расширения |
| `CPU` | `oc2r:devices/cpu` | Процессор |
| `ROBOT_MODULE` | `oc2r:devices/robot_module` | Модуль робота |
| `FLOPPY` | `oc2r:devices/floppy` | Дискета |
| `NETWORK_TUNNEL` | `oc2r:devices/network_tunnel` | Туннельная карта |

## Связанные материалы

- [Архитектура OC2R](https://github.com/DivByDiamond/OC2R/blob/39eb8109e2e38fdb047af98d8e6f7ded0194f498/docs/ARCHITECTURE.md): граф шины и жизненный цикл виртуальной машины.
- [Структура исходников](https://github.com/DivByDiamond/OC2R/blob/39eb8109e2e38fdb047af98d8e6f7ded0194f498/docs/SRC_STRUCTURE.md).
- [Машинный интерфейс](/ru/block/machine_interface): готовое устройство для работы с механизмами.
