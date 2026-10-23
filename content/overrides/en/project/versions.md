source: docs/roadmap/multiloader.md
source_sha256: 154a14b1e589d972eb264ec7b0375e8b4f59b8fc3cbdd560e518cad752e9ffe1
# Minecraft versions and loaders

This page describes the build and verification targets recorded in the OC2R source revision linked below. Choose a mod artifact matching both your Minecraft version and loader.

## Target matrix

| Minecraft | NeoForge | Fabric |
|---|---|---|
| 1.21.1 | Build and verification target | Build and verification target |
| 26.1.2 | Build and verification target | Future work; no build node |
| 26.2 | Build and verification target | Future work; no build node |

The source records four targets, not every combination of the three Minecraft versions and two loaders. The `settings.gradle.kts` build includes three NeoForge nodes and only the 1.21.1 Fabric node. The 26.x builds use a Java 25 toolchain; the 1.21.1 builds use Java 21.

## Release availability

The release workflow is configured for those four artifacts. Platform uploads depend on the repository's platform settings and on artifacts being built. A configured build target does not guarantee that a particular release is already downloadable on every platform.

Check the Minecraft version and loader shown on the release file before installing it. Keep the mod jar in the instance's `mods` directory. Fabric also requires its declared dependencies: Fabric API, Team Reborn Energy, and Forge Config API Port.

## Reading development documents

The older `docs/MULTILOADER.md` describes the original six-target design. Use the current roadmap and build configuration to distinguish implemented targets from future work. Feature behavior can also depend on server configuration; this wiki is a snapshot of the linked source revision.
