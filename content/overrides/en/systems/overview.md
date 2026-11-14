source: docs/ONYXOS.md
source_sha256: f44044aa8c5102410b9525b81b2b74d0894e5fa9455c0dcf9d0972114e679be7
# Operating systems

A computer's firmware selects its operating system. OC2R includes Linux (Minux / Buildroot) and OnyxOS. Power the computer off before changing firmware.

## Linux

Start with [the quickstart guide](/en/getting_started) to boot Linux and log in as `root`. Linux provides Lua and the `devices` library used in the [scripting guide](/en/scripting).

Firmware loads the kernel from the built-in root disk. A separate hard drive provides additional storage; it is not required for the standard boot. See [Buildroot](/en/systems/buildroot) for image builds, 9p, and `/mnt/builtin`.

## OnyxOS

OnyxOS has its own kernel, filesystem, and userspace. Linux commands and Lua examples do not automatically apply to it. The bundled image provides [oc2ctl for device RPC](/en/reference/onyx-rpc).

Log in as `root` with an empty password. Four 16 MiB memory modules provide the tested 64 MiB configuration, not a measured minimum for every application. Read the [image documentation](/en/systems/onyxos-reference) for the precise build and supported behavior.

The image also contains the build for the [tablet](/en/item/tablet), the `rshd` remote shell for access from a PC through the [VXLAN hub](/en/reference/vxlan), and `play`, a terminal player for DFPWM music on a [speaker](/en/block/speaker) or a [sound card](/en/item/sound_card). It was rebuilt on 14 November 2026.

## Saved roots

Linux and OnyxOS retain separate root overlays. Changing firmware does not migrate files. Updating the bundled image does not install new tools into existing private roots.

Back up `config/oc2r/root-images/` with your world, especially when overriding the Onyx root image. Existing roots retain their original base image identity.
