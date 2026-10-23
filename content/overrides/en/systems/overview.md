source: docs/ONYXOS.md
source_sha256: 579be4a8cb9d3806d64bdea13395668fc3f22b0d33754c537f94ea5331991dd8
# Operating systems

A computer's firmware selects its operating system. OC2R includes Linux (Minux / Buildroot) and OnyxOS. Power the computer off before changing firmware.

## Linux

Start with [the quickstart guide](/en/getting_started) to boot Linux and log in as `root`. Linux provides Lua and the `devices` library used in the [scripting guide](/en/scripting).

Firmware loads the kernel from the built-in root disk. A separate hard drive provides additional storage; it is not required for the standard boot. See [Buildroot](/en/systems/buildroot) for image builds, 9p, and `/mnt/builtin`.

## OnyxOS

OnyxOS has its own kernel, filesystem, and userspace. Linux commands and Lua examples do not automatically apply to it. The bundled image provides [oc2ctl for device RPC](/en/reference/onyx-rpc).

Log in as `root` with an empty password. Four 16 MiB memory modules provide the tested 64 MiB configuration, not a measured minimum for every application. Read the [image documentation](/en/systems/onyxos-reference) for the precise build and supported behavior.

## Saved roots

Linux and OnyxOS retain separate root overlays. Changing firmware does not migrate files. Updating the bundled image does not install new tools into existing private roots.

Back up `config/oc2r/root-images/` with your world, especially when overriding the Onyx root image. Existing roots retain their original base image identity.
