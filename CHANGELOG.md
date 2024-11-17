# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Ideas]

- clap CLI to validate configuration files (main settings, biomes)
- generation steps

## [Unrealeased]

## [0.1.5] - 2024-11-17

### Changed

- Big refacto following Bevy recommandations for plugin architecture
    - WorldsPlugin is composed by the camera, UI, settings and the map
    - MapPlugin do everything else mainly generating, rendering the tilemap
- Use bevy_ecs_tilemap for handling tilemap in ECS way (not sure yet if its the more accurate choice for generating worlds)
- Enable/Disable map "layer" (elevation, temperature)
    - There is only one layer, rendering "layers" for now is just mixing their colors
- Hexagonal tiles ! I plan to project them on a sphere

### Added

- MapSet systemset: Prepare, Generate, Render are chained in this order

## [0.1.4] - 2024-10-31

### Added

- Map Modes: Elevation, Temperature, Shapes

## [0.1.3] - 2024-10-28

### Added

- Offset on noise maps, another parameter to randomize render
- Sea level parameter to determine the elevation where end ocean and start land
- Enable/Disable biome in their definition

### Fixed

- Tilemap is now updated at DrawMapEvent, and not recreated which led to stacking them
- Absorbing input events on Egui window to avoid moving the map as well

## [0.1.2] - 2024-10-28

### Added

- Cargo.lock is now versionned

### Fixed

- Release workflow (profile release-native was missing)

## [0.1.1] - 2024-10-28

### Added

- Add basic CI worklows, to validate PR and release a tag
- Biome mod for loading and testing them
- First biomes and their settings

### Changed

- Use of bevy fast tilemap instead of bevy ecs tilemap, to gain performance

## [0.1.0] - 2024-10-22

### Generated tiled-base map

First try:

1. Used a noise algorithm (Perlin for now, but i have to look at Simplex (OpenSimplex, SuperSimplex)) to build a noise map.
2. Add many octaves to have a more realistic terrain.
    * Add lacunarity and persistance which increase powerly by the number of octave

### Added

- A first version of the bevy plugin that looks more like POC
- A Config struct containing every variables required to generate the map
- This CHANGELOG file
- A tiny README

