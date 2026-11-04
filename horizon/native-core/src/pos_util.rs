// MIT ReTerraForged PosUtil.java, preservation baseline 29239f1f164d19dbfcccaca4a8277d64e784207c.
#[inline]
pub fn pack(left: i32, right: i32) -> i64 {
    ((right as u32 as u64) | ((left as u32 as u64) << 32)) as i64
}
#[inline]
pub fn pack_float_cast(left: f32, right: f32) -> i64 {
    // Java casts float to long before retaining its low 32 bits here.
    (((right as i64 as u64) & 0xffffffff) | (((left as i64 as u64) & 0xffffffff) << 32)) as i64
}
#[inline]
pub fn pack_floats(left: f32, right: f32) -> i64 {
    (right.to_bits() as u64 | ((left.to_bits() as u64) << 32)) as i64
}
#[inline]
pub fn pack_int_float(left: i32, right: f32) -> i64 {
    pack(left, right.to_bits() as i32)
}
#[inline]
pub fn pack_float_int(left: f32, right: i32) -> i64 {
    pack(left.to_bits() as i32, right)
}
#[inline]
pub fn unpack_left(packed: i64) -> i32 {
    ((packed as u64) >> 32) as i32
}
#[inline]
pub fn unpack_right(packed: i64) -> i32 {
    packed as i32
}
#[inline]
pub fn unpack_left_float(packed: i64) -> f32 {
    f32::from_bits(unpack_left(packed) as u32)
}
#[inline]
pub fn unpack_right_float(packed: i64) -> f32 {
    f32::from_bits(unpack_right(packed) as u32)
}
#[inline]
pub fn contains_i32(x: i32, z: i32, x1: i32, z1: i32, x2: i32, z2: i32) -> bool {
    x >= x1 && x < x2 && z >= z1 && z < z2
}
#[inline]
pub fn contains_f32(x: f32, z: f32, x1: f32, z1: f32, x2: f32, z2: f32) -> bool {
    x >= x1 && x < x2 && z >= z1 && z < z2
}
