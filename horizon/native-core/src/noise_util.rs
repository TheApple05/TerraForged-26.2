// Portions of this module are a mechanical translation of public scalar/hash
// methods from MIT-licensed ReTerraForged NoiseUtil.java
// (raccoonman/reterraforged, preservation baseline
// 29239f1f164d19dbfcccaca4a8277d64e784207c). MIT license preserved upstream.
// This isolated math module is not activated in Horizon terrain generation.

#[inline]
pub fn map_clamped_range(value: f32, min: f32, max: f32, range: f32) -> f32 {
    let clamped = clamp_f32(value, min, max);
    let dif = clamped - min;
    if dif >= range { 1.0f32 } else { dif / range }
}

#[inline]
pub fn map_f32(value: f32, from: f32, to: f32, min: f32, max: f32) -> f32 {
    let alpha = (value - min) / (max - min);
    from + alpha * (to - from)
}

#[inline]
pub fn map_f64(value: f64, from: f64, to: f64, min: f64, max: f64) -> f64 {
    let alpha = (value - min) / (max - min);
    from + alpha * (to - from)
}

#[inline]
pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[inline]
pub fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[inline]
pub fn dist2(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy
}

#[inline]
pub fn dot(x0: f32, y0: f32, x1: f32, y1: f32) -> f32 {
    x0 * x1 + y0 * y1
}

#[inline]
pub fn div(num: i32, denom: i32) -> f32 {
    num as f32 / denom as f32
}

#[inline]
pub fn floor_legacy(f: f32) -> i32 {
    if f >= 0.0f32 {
        f as i32
    } else {
        (f as i32).wrapping_sub(1)
    }
}

#[inline]
pub fn to_int(f: f32) -> i32 {
    let i = f.to_bits() as i32;
    i ^ (i >> 16)
}

#[inline]
pub fn round_legacy(f: f32) -> i32 {
    if f >= 0.0f32 {
        (f + 0.5f32) as i32
    } else {
        (f - 0.5f32) as i32
    }
}

#[inline]
pub fn lerp_f32(a: f32, b: f32, alpha: f32) -> f32 {
    a + alpha * (b - a)
}

#[inline]
pub fn lerp_f64(a: f64, b: f64, alpha: f64) -> f64 {
    a + alpha * (b - a)
}

#[inline]
pub fn lerp_map(alpha_value: f32, alpha_min: f32, alpha_max: f32, from: f32, to: f32) -> f32 {
    let alpha = (alpha_value - alpha_min) / (alpha_max - alpha_min);
    from + alpha * (to - from)
}

#[inline]
pub fn interp_hermite(t: f32) -> f32 {
    t * t * (3.0f32 - 2.0f32 * t)
}

#[inline]
pub fn interp_quintic(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0f32 - 15.0f32) + 10.0f32)
}

#[inline]
pub fn curve(t: f32, steepness: f32) -> f32 {
    curve_mid(t, 0.5f32, steepness)
}

#[inline]
pub fn curve_mid(t: f32, mid: f32, steepness: f32) -> f32 {
    1.0f32 / (1.0f32 + exp_approx(-steepness * (t - mid)))
}

#[inline]
pub fn cubic_lerp(a: f32, b: f32, c: f32, d: f32, t: f32) -> f32 {
    let p = d - c - (a - b);
    t * t * t * p + t * t * (a - b - p) + t * (c - a) + b
}

#[inline]
pub fn exp_approx(mut x: f32) -> f32 {
    x = 1.0f32 + x / 256.0f32;
    x *= x;
    x *= x;
    x *= x;
    x *= x;
    x *= x;
    x *= x;
    x *= x;
    x *= x;
    x
}

#[inline]
pub fn copy_sign_legacy(value: f32, sign: f32) -> f32 {
    if sign < 0.0f32 && value > 0.0f32 {
        return -value;
    }
    if sign > 0.0f32 && value < 0.0f32 {
        return -value;
    }
    value
}

#[inline]
pub fn pow_i32(value: f32, power: i32) -> f32 {
    if power == 0 {
        return 1.0f32;
    }
    if power == 1 {
        return value;
    }
    if power == 2 {
        return value * value;
    }
    if power == 3 {
        return value * value * value;
    }
    if power == 4 {
        return value * value * value * value;
    }
    let mut result = 1.0f32;
    let mut i = 0i32;
    while i < power {
        result *= value;
        i = i.wrapping_add(1);
    }
    result
}

#[inline]
pub fn hash(x: i32, y: i32) -> i32 {
    let mut hash = x;
    hash ^= 31337i32.wrapping_mul(y);
    hash = hash
        .wrapping_mul(hash)
        .wrapping_mul(hash)
        .wrapping_mul(60493);
    hash ^= hash >> 13;
    hash
}

#[inline]
pub fn hash_2d(seed: i32, x: i32, y: i32) -> i32 {
    let mut hash = seed;
    hash ^= 1619i32.wrapping_mul(x);
    hash ^= 31337i32.wrapping_mul(y);
    hash = hash
        .wrapping_mul(hash)
        .wrapping_mul(hash)
        .wrapping_mul(60493);
    hash ^= hash >> 13;
    hash
}

#[inline]
pub fn val_coord_2d(seed: i32, x: i32, y: i32) -> f32 {
    let mut n = seed;
    n ^= 1619i32.wrapping_mul(x);
    n ^= 31337i32.wrapping_mul(y);
    n.wrapping_mul(n).wrapping_mul(n).wrapping_mul(60493) as f32 / 2.14748365E9f32
}
