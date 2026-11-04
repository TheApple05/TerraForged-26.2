/*
 * MIT License. Copyright (c) 2020-2021 TerraForged.
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
//! Original classic pillar mask, legacy terrace and bounded height arithmetic.
//! Block access, biome boundaries and material placement are separate integration.
use crate::graph::{java_max, java_min};
fn clamp(value: f32) -> f32 {
    java_max(0.0, java_min(1.0, value))
}
pub fn mask(biome_region_edge: f32, river_mask: f32, gradient: f32) -> f32 {
    clamp(biome_region_edge) * clamp(river_mask / 0.0005) * clamp(1.0 - gradient)
}
pub fn terrace(input: f32, maximum: f32) -> f32 {
    let value = clamp(input);
    let spacing = maximum / 3.0;
    let index = (value * 3.0 + 0.5) as i32;
    let step = index as f32 * spacing;
    let bound = (spacing - spacing * 0.25) / 2.0;
    if value < step - bound && index > 0 {
        let lower = (index - 1) as f32 * spacing;
        let alpha = (value - (lower + bound)) / ((step - bound) - (lower + bound));
        let alpha = 1.0 - curve(alpha);
        step - alpha * (step - lower) * 0.25
    } else if value > step + bound && index < 3 {
        let upper = (index + 1) as f32 * spacing;
        let alpha = (value - (step + bound)) / ((upper - bound) - (step + bound));
        step + curve(alpha) * (upper - step)
    } else {
        step
    }
}
pub fn added_height(shape: f32, mask: f32, stone_forest: bool) -> i32 {
    (clamp(shape) * clamp(mask) * if stone_forest { 50.0 } else { 30.0 }) as i32
}
fn curve(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}
