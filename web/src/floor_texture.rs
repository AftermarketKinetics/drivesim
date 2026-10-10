//! The cage floor's look: painted plywood with its 4 × 8 ft sheet seams,
//! a mottled paint finish and scuffs, generated at startup. A flat grey
//! floor gave no cue of motion over it, so rolling read as sliding (user,
//! 2026-10-10: "everything is slidey").

use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use drivesim_physics::arena::ARENA_SIDE_M;

/// Texels along each side; the texture covers the whole floor once
/// (~4.8 mm per texel on the 4.88 m cage).
pub const SIZE: usize = 1024;
/// A plywood sheet, m: 8 ft along x (away from the driver), 4 ft across.
const SHEET_X_M: f32 = 2.4384;
const SHEET_Y_M: f32 = 1.2192;
/// Scuff marks: tyre rubber (dark) and scraped paint (light).
const SCUFFS: usize = 90;

/// splitmix64: a fixed seed gives the same floor every load.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// Smooth value noise in 0..1 with cells `cell` texels wide.
fn value_noise(cell: usize, seed: u64) -> Vec<f32> {
    let n = SIZE / cell + 2;
    let mut rng = Rng(seed);
    let grid: Vec<f32> = (0..n * n).map(|_| rng.next()).collect();
    let mut out = vec![0.0; SIZE * SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (gx, gy) = (x / cell, y / cell);
            let (fx, fy) = (
                (x % cell) as f32 / cell as f32,
                (y % cell) as f32 / cell as f32,
            );
            let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
            let g = |i: usize, j: usize| grid[j * n + i];
            let top = g(gx, gy) + (g(gx + 1, gy) - g(gx, gy)) * sx;
            let bottom = g(gx, gy + 1) + (g(gx + 1, gy + 1) - g(gx, gy + 1)) * sx;
            out[y * SIZE + x] = top + (bottom - top) * sy;
        }
    }
    out
}

/// Brightness of each texel, 0..1 (linear-ish paint value).
fn shade() -> Vec<f32> {
    let m_per_px = ARENA_SIDE_M / SIZE as f32;
    let mottle = value_noise(64, 1);
    let grain = value_noise(4, 2);
    let mut v: Vec<f32> = mottle
        .iter()
        .zip(&grain)
        .map(|(m, g)| 0.30 + 0.035 * (m - 0.5) + 0.02 * (g - 0.5))
        .collect();
    // Sheet seams: a dark line ~1 cm wide every sheet edge.
    for (i, px) in v.iter_mut().enumerate() {
        let (x, y) = ((i % SIZE) as f32 * m_per_px, (i / SIZE) as f32 * m_per_px);
        let near = |p: f32, pitch: f32| {
            let d = p % pitch;
            d.min(pitch - d) < 0.005
        };
        if near(x, SHEET_X_M) || near(y, SHEET_Y_M) {
            *px *= 0.55;
        }
    }
    // Scuffs: short curved strokes.
    let mut rng = Rng(3);
    for _ in 0..SCUFFS {
        let (mut x, mut y) = (rng.next() * SIZE as f32, rng.next() * SIZE as f32);
        let mut heading = rng.next() * std::f32::consts::TAU;
        let curl = (rng.next() - 0.5) * 0.08;
        let len = 20 + (rng.next() * 110.0) as usize;
        let width = 1.0 + rng.next() * 2.5;
        let dark = rng.next() < 0.65;
        let strength = 0.15 + rng.next() * 0.25;
        for _ in 0..len {
            heading += curl;
            x += heading.cos();
            y += heading.sin();
            let r = width.ceil() as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    let (px, py) = (x as i32 + dx, y as i32 + dy);
                    if px < 0 || py < 0 || px >= SIZE as i32 || py >= SIZE as i32 {
                        continue;
                    }
                    if ((dx * dx + dy * dy) as f32).sqrt() > width {
                        continue;
                    }
                    let t = &mut v[py as usize * SIZE + px as usize];
                    *t = if dark {
                        *t * (1.0 - strength)
                    } else {
                        *t + (0.6 - *t) * strength
                    };
                }
            }
        }
    }
    v
}

/// The floor texture with a full mip chain (Bevy does not build mips for
/// runtime images, and the driver sees the floor at a grazing angle, where
/// an unmipped texture shimmers).
pub fn floor_image() -> Image {
    let mut level: Vec<u8> = shade()
        .into_iter()
        .flat_map(|s| {
            let c = (s.clamp(0.0, 1.0) * 255.0) as u8;
            [c, c, c.saturating_add(2), 255]
        })
        .collect();
    let mut data = level.clone();
    let (mut side, mut mips) = (SIZE, 1u32);
    while side > 1 {
        let half = side / 2;
        let mut next = vec![0u8; half * half * 4];
        for y in 0..half {
            for x in 0..half {
                for c in 0..4 {
                    let at = |xx: usize, yy: usize| level[(yy * side + xx) * 4 + c] as u32;
                    let sum = at(2 * x, 2 * y)
                        + at(2 * x + 1, 2 * y)
                        + at(2 * x, 2 * y + 1)
                        + at(2 * x + 1, 2 * y + 1);
                    next[(y * half + x) * 4 + c] = (sum / 4) as u8;
                }
            }
        }
        data.extend_from_slice(&next);
        level = next;
        side = half;
        mips += 1;
    }
    let size = Extent3d {
        width: SIZE as u32,
        height: SIZE as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_uninit(
        size,
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.mip_level_count = mips;
    image.data = Some(data);
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        address_mode_v: ImageAddressMode::ClampToEdge,
        anisotropy_clamp: 16,
        ..ImageSamplerDescriptor::linear()
    });
    image
}
