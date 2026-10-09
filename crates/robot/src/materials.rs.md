# materials.rs

## Function

Material names in robot.toml → densities.

## Interface

`density(name) -> Option<f64>`, kg/m³; names are case- and
space-insensitive (`"Mild Steel"` = `mild-steel`).

## Implementation

Follows the cad-step plugin's table (g/cm³ × 1000) and adds AR500 /
Hardox (7.85). A density the table lacks goes in robot.toml as
`density_kg_m3` on a material rule instead.

## Assertions

- Unknown names are `None`, never a default density.
