source: docs/TABLET.md
source_sha256: 2dcd9918fb2362f18750f61573f0d6258e0bf34c5aa46fb212201d3a30aa7f7e
# Tablet
`oc2r:tablet`

The tablet is a pocket computer: an item that holds a small virtual machine, running OnyxOS by default. It is the same machine model as a [computer](/en/block/computer) or a [robot](/en/item/robot), with less power, one built-in screen and no cables. It is meant for small graphical tools such as dashboards, a messenger or simple programs written by players. Heavy work stays on PCs, and the tablet talks to them over the [ORA radio network](/en/reference/ora-radio).

## Hardware

- One processor slot, one flash slot and one optional hard drive slot, edited from the inventory button of the tablet screen.
- RAM is fixed at 16 MiB and has no slot. The processor clock is capped to a quarter of the computer clock whatever processor is inserted.
- The tablet uses no energy.
- A new tablet comes with an OnyxOS flash. Without a flash it reports missing firmware and does not boot.

## Using it

Right click with the tablet in hand to open its screen. The screen uses the same frame as the computer and the monitor: a sidebar with power, input capture, inventory and a display button that switches between two views of the same running machine.

- **Screen** shows a 256 × 160 graphical framebuffer (RGB565). Keyboard and mouse go to the built-in input device with absolute pointer coordinates.
- **Console** shows the UART terminal and routes typing to it. Clipboard paste works in the console.

Switching views keeps the running application. Input capture is released with the same single key as for the monitor and the keyboard (Right Ctrl by default, rebindable). Closing the screen, changing views or losing focus also releases held input.

To try the graphics, open the Console, log in as `root` and run `otablet --device`, then switch to Screen. The demo shows labels, a counter button and a progress bar. Press Q, Escape or right click to leave it.

## Lifetime and saving

Each tablet has an id stored in the item, and its machine state is saved in the world under that id (`<world>/data/oc2r_tablets/<uuid>.dat`). The machine runs only while the tablet is held or its screen is open. Two seconds after that it is saved and stopped, so dropping the item, dying or logging out lose nothing. While running it is also written every five minutes, so a server crash loses at most that much guest time. A tablet that merely sits in an inventory does not run.

## Network

The tablet is one more radio device. It attaches to an access point in range (a [GPS tower](/en/block/gps_tower) within 45 blocks, or a [robot dock](/en/block/charger) or [wireless access point](/en/block/wireless_access_point) within 10 blocks), is admitted by an invitation that the network administrator approves from a PC, and is then addressed by an ORA address such as `7K3F:02Q9`.

In the guest the tablet uses the same device bus as a computer: JSON RPC on `/dev/hvc0`. The `tablet` device exposes `getRadioEndpointId`, `joinRadioNetwork`, `getRadioAddress`, `getRadioStatus`, messaging, service discovery, telemetry, commands and ping. To enrol:

1. Call `getRadioEndpointId()` on the tablet.
2. On the administrator's PC, issue an approved invitation for that endpoint.
3. Call `joinRadioNetwork(invite, secret)` on the tablet.

The `ochat` and `ochat-gui` messengers already work this way. The device also advertises a `gps_tower` alias so `ochat` finds it; it provides radio calls, not GPS positioning.

The tablet's credential lives in a server-only item component and is never sent to the client. A second copy of the item cannot use the active tablet's terminal, inventory or radio, and every activation rotates the credential, so copies are not independent computers. Radio disconnects as soon as no player holds the item. Enrolment is bound to a dimension: roaming inside the network keeps the ORA address, another dimension cannot use the old enrolment, and the local terminal stays usable when the radio is revoked or out of range.

## Reflashing

The operating system lives on the flash item. Take the flash out of the tablet, put it in the flash writer (it offers the bundled OnyxOS and Minux images and a file chooser for your own image), put it back and turn the tablet on.

## Writing software

- **C programs**: C99 programs built with OnyxCC from the OnyxApps repository. Use `lib/oc2rpc` (`oc2_open`, `oc2_call`) for radio and RPC, with `apps/ochat` as a worked example, and the framebuffer and mouse frontend from `apps/otablet` for labels, buttons and progress bars. Only one process should own `/dev/hvc0` at a time. Programs run in the user ring.
- **Lua**: OnyxOS ships Lua 5.5 (`lua`, `luac`). Run scripts from the shell with `lua script.lua`. Lua has no radio or RPC binding yet.
- **Your own OS**: anything that boots like OnyxOS can run on the tablet: OpenSBI jump firmware followed by a flat RISC-V binary with the S-mode payload at `0x80200000`, a virtio block root disk and a virtio console as the device bus. See [OnyxOS](/en/systems/onyxos-reference).

The display is a `simple-framebuffer` device (256 × 160, RGB565 little endian, 512-byte stride). The built-in VirtIO input advertises absolute X/Y ranges 0–255 and 0–159, Linux evdev keyboard codes and three pointer buttons.
