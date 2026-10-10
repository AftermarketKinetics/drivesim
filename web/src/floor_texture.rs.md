# floor_texture.rs

## Function

Paints the cage floor: dead-flat grey paint over plywood, with the
4 × 8 ft sheet seams, a mottled finish and scuffs. Plain grey gave no
sense of motion over the floor, and rolling read as sliding (user,
2026-10-10: "everything is slidey").

## Interface

- `floor_image() -> Image`: a `SIZE` × `SIZE` (1024) sRGB texture with a
  full mip chain and its own clamped, 16× anisotropic sampler. It covers
  the whole floor once, about 4.8 mm per texel.
- Used by `arena_render.rs` as the floor material's
  `base_color_texture`.

## Implementation

- **Generated, not loaded:** the app stays a single wasm, with no asset
  to fetch or keep in step. A fixed-seed splitmix PRNG gives the same
  floor on every load.
- **The paint** is value noise at two scales: 64 texel mottle, ±3.5%,
  and 4 texel grain, ±2%, around 0.30.
- **Seams:** a line ~1 cm wide at every sheet edge, darkened to 55%.
  Sheets are 8 ft along x (away from the driver) and 4 ft across.
- **Scuffs:** 90 short curved strokes; 65% are tyre rubber (darker), the
  rest scraped paint (lighter).
- **Mips are built here** by 2 × 2 box filtering, appended level after
  level as Bevy uploads them. Bevy makes no mips for runtime images, and
  the driver sees the floor at a grazing angle, where an unmipped
  texture shimmers.

## Assertions

- Same output on every run: the RNG is seeded, never from time.
- `mip_level_count` equals the number of levels in `data`.
