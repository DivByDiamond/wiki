source: docs/roadmap/multiloader.md
source_sha256: 7ba4f45a508c8becc9d7b32850f8ac8be0900d480d819aa3cf01fedefdae75bd
# Minecraft versions and loaders

The current OC2R source has six build targets. Choose an artifact matching both your Minecraft version and loader.

## Implemented targets

| Minecraft | NeoForge | Fabric | Java |
|---|---|---|---|
| 1.21.1 | Implemented | Implemented | 21 |
| 26.1.2 | Implemented | Implemented | 25 |
| 26.2 | Implemented | Implemented | 25 |

Fabric 26.1.2 and 26.2 have been accepted, merged and verified in CI. The earlier statement that Fabric 26.x was future work is outdated. Local acceptance included server tests and client runs on both modern versions.

## Download availability

Check the artifacts in [OC2R releases](https://github.com/DivByDiamond/OC2R/releases). An implemented build target does not establish publication on every distribution platform. In the source revision linked below, publication of the new Fabric 26.x targets is a separate step deferred at the owner's request.

NeoForge and Fabric artifacts are not interchangeable. Fabric also requires compatible Fabric API, Team Reborn Energy and Forge Config API Port versions declared for the selected artifact.

## Reading the acceptance reports

Dated roadmap sections preserve earlier limitations. The current summary records all six targets as implemented. Use the current acceptance report for implementation status and the release page for downloadable artifacts.

The [Fabric 26.x acceptance report](https://github.com/DivByDiamond/OC2R/blob/39eb8109e2e38fdb047af98d8e6f7ded0194f498/docs/roadmap/fabric-26-acceptance.md) contains the target matrix and validation results. Server settings may change hardware characteristics.
