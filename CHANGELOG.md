# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Upcoming]

* Increase performance by droping tiles and using meshes instead. On group of meshes per layer should be a good start.
  Steps to make such a meshes based map:
  1. Still iterates over each point of the grid (may not be the better approach, can i cluster directly from a noise map ?)
  2. Scale each point to the number of layer and push the point to a Layer (vector, list of points)
  3. For each Layer use a `linfa_clustering` algorithm to make a group of cluster
  4. For each cluster use Hull Convex algorithm (from `geo` crate) to compute the outline path of the cluster and make a group of Polyline
  5. For each polyline, create a Mesh with the proper layer attributes (eg. color)
  6. Tesselate and render (with `bevy_prototype_lyon`) meshes

## [Unreleased]

## [0.1.0] - 2024-10-22

### Generated tiled-base map

First  try:

1. Used a noise algorithm (Perlin for now, but i have to look at Simplex (OpenSimplex, SuperSimplex)) to build a noise map.
2. Add many octaves to have a more realistic terrain.
    * Add lacunarity and persistance which increase powerly by the number of octave

### Added

- A first version of the bevy plugin that looks more like POC
- A Config struct containing every variables required to generate the map
- This CHANGELOG file
- A tiny README

