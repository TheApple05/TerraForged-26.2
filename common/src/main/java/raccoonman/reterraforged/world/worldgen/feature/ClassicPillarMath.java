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
package raccoonman.reterraforged.world.worldgen.feature;

/** Original MaskedSurface and Noise2D LegacyTerrace equations, independent of game APIs. */
public final class ClassicPillarMath {
    private ClassicPillarMath() {}

    public static float mask(float biomeRegionEdge, float riverMask, float gradient) {
        return clamp(biomeRegionEdge) * clamp(riverMask / 0.0005F) * clamp(1.0F - gradient);
    }

    /** Noise2D legacyTerrace(1, 0.25, 4, 0.25); not the newer Terrace node. */
    public static float terrace(float input, float maximum) {
        float value = clamp(input);
        float spacing = maximum / 3;
        int index = (int) (value * 3 + 0.5F);
        float step = index * spacing;
        float bound = (spacing - spacing * 0.25F) / 2;
        if (value < step - bound && index > 0) {
            float lower = (index - 1) * spacing;
            float alpha = (value - (lower + bound)) / ((step - bound) - (lower + bound));
            alpha = 1 - curve(alpha);
            return step - alpha * (step - lower) * 0.25F;
        } else if (value > step + bound && index < 3) {
            float upper = (index + 1) * spacing;
            float alpha = (value - (step + bound)) / ((upper - bound) - (step + bound));
            return step + curve(alpha) * (upper - step);
        }
        return step;
    }

    public static int addedHeight(float shape, float mask, boolean stoneForest) {
        return (int) (clamp(shape) * clamp(mask) * (stoneForest ? 50 : 30));
    }

    private static float curve(float value) { return value * value * (3.0F - 2.0F * value); }
    private static float clamp(float value) { return Math.max(0.0F, Math.min(1.0F, value)); }
}
