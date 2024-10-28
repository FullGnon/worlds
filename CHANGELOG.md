# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

