source: docs/PERIPHERALS.md
source_sha256: 2d7d3634607678e40a830bb76caef2b5e8f93d4252893c6422205116f08766cc
# Environment sensor
`oc2r:environment_sensor`

A wall-mounted sensor that reports the weather, light and biome around its own block to a computer. It appears as the `environment_sensor` device on `/dev/hvc0`. No network card is needed.

## Mounting

The sensor rotates horizontally and its rear sits against the support. Mount it directly on the outward face of a device-bus cable and put a [bus interface](/en/block/bus_interface) on that cable face. Only the rear accepts RPC; the front, sides, top and bottom do not. It can also be mounted on a solid wall as a purely decorative, disconnected sensor. Side interfaces from older versions stay in place as cable parts but no longer expose the sensor, so move them to its rear.

## RPC

- `getEnvironment()` returns a read-only snapshot: biome id, biome temperature, sky light, block light, local rain, global thunder, sky visibility, day time and the sample game tick.
- `getSampleIntervalTicks()` returns 20. Calls inside that interval reuse the cached sample.
- `getUnits()` describes the values: light is 0–15, time is in game ticks and temperature is Minecraft's biome temperature scalar. That scalar is not degrees Celsius.
- The `environment_changed` bus event reports a change in the sampled biome, light, weather, sky visibility or temperature. Its payload holds `sampledAt`; call `getEnvironment()` for the new values. Time passing alone does not produce an event every second.

## Behaviour

Sampling reads only the sensor's own loaded block and loads no neighbouring chunks. Automatic sampling and the "on" texture need a mounted connection to a running VM; a stopped computer or an unattached bus leaves the sensor dark. Sampling and events stop when it unloads, and a controller's event subscriptions are cleared when it disconnects.
