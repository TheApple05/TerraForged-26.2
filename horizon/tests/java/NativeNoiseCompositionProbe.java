package raccoonman.reterraforged.world.worldgen.noise.module;

import java.util.ArrayList;
import java.util.List;

import raccoonman.reterraforged.world.worldgen.noise.function.Interpolation;

/**
 * Developer-only independent oracle probe. Constructs genuine Noise graphs,
 * prints legacy min/max metadata and raw compute outputs as hex bit patterns.
 * Rust actual values are compared with these frozen Java observations; this file
 * only observes genuine Java behavior.
 */
public final class NativeNoiseCompositionProbe {
	private static final float[] XS = {
		-3.0E7F, -17.0F, -16.25F, -16.0F, -1.5F, -1.0F, -0.25F,
		-0.0F, 0.0F, 0.25F, 1.0F, 15.0F, 15.75F, 16.0F, 16.25F, 17.0F, 3.0E7F
	};
	private static final int[] SEEDS = {
		0, 1, -1, 42, Integer.MIN_VALUE, Integer.MAX_VALUE
	};

	private record NamedGraph(String name, Noise noise) {
	}

	private NativeNoiseCompositionProbe() {
	}

    public static void main(String[] args) throws Exception {
		List<NamedGraph> graphs = buildGraphs();
		StringBuilder out = new StringBuilder(1 << 20);

		for (int i = 0; i < graphs.size(); i++) {
			NamedGraph g = graphs.get(i);
			out.append("graph/").append(i).append('\t')
				.append(g.name()).append(',')
				.append(hex(g.noise().minValue())).append(',')
				.append(hex(g.noise().maxValue())).append('\n');
		}

		for (int i = 0; i < graphs.size(); i++) {
			Noise noise = graphs.get(i).noise();
			for (int seed : SEEDS) {
				for (int xi = 0; xi < XS.length; xi++) {
					for (int zi = 0; zi < XS.length; zi++) {
						float value = noise.compute(XS[xi], XS[zi], seed);
						if (!Float.isFinite(value)) {
							throw new IllegalStateException(
								"non-finite compute result graph=" + i + " seed=" + seed + " xi=" + xi + " zi=" + zi + " value=" + value);
						}
						out.append("sample/").append(i).append('/').append(seed).append('/')
							.append(xi).append('/').append(zi).append('\t')
							.append(hex(value)).append('\n');
					}
				}
			}
		}

		if (args.length == 1) {
			java.nio.file.Files.writeString(java.nio.file.Path.of(args[0]), out.toString(), java.nio.charset.StandardCharsets.UTF_8);
		} else {
			System.out.print(out);
		}
	}

	private static String hex(float value) {
		return String.format("%08x", Float.floatToRawIntBits(value));
	}

	private static Noise simplex() {
		return new Simplex(0.0125F, 3, 2.0F, 0.5F, Interpolation.CURVE3);
	}

	private static Noise perlin() {
		return new Perlin2(12345, 0.02F, 2, 2.0F, 0.5F, Interpolation.CURVE4);
	}

	private static Noise c(float value) {
		return new Constant(value);
	}

	private static Noise add(Noise a, Noise b) {
		return new Add(a, b);
	}

	private static Noise mul(Noise a, Noise b) {
		return new Multiply(a, b);
	}

	private static Noise abs(Noise a) {
		return new Abs(a);
	}

	private static Noise invert(Noise a) {
		return new Invert(a);
	}

	private static Noise min(Noise a, Noise b) {
		return new Min(a, b);
	}

	private static Noise max(Noise a, Noise b) {
		return new Max(a, b);
	}

	private static Noise clamp(Noise input, Noise lo, Noise hi) {
		return new Clamp(input, lo, hi);
	}

	private static Noise map(Noise alpha, Noise from, Noise to) {
		return new Map(alpha, from, to);
	}

	private static Noise shift(Noise input, int shift) {
		return new ShiftSeed(input, shift);
	}

	private static Noise freq(Noise input, Noise xFreq, Noise zFreq) {
		return new Frequency(input, xFreq, zFreq);
	}

	private static List<NamedGraph> buildGraphs() {
		List<NamedGraph> graphs = new ArrayList<>();

		graphs.add(new NamedGraph("constant-poszero", c(0.0F)));
		graphs.add(new NamedGraph("constant-negzero", c(-0.0F)));
		graphs.add(new NamedGraph("constant-1", c(1.0F)));
		graphs.add(new NamedGraph("constant-neg1", c(-1.0F)));
		graphs.add(new NamedGraph("constant-neg2", c(-2.0F)));
		graphs.add(new NamedGraph("constant-3", c(3.0F)));

		graphs.add(new NamedGraph("simplex-base", simplex()));
		graphs.add(new NamedGraph("perlin-base", perlin()));
		graphs.add(new NamedGraph("simplex-negfreq", freq(simplex(), c(-1.5F), c(0.75F))));
		graphs.add(new NamedGraph("perlin-varfreq", freq(perlin(), add(c(0.5F), mul(simplex(), c(0.5F))), c(-0.25F))));
		graphs.add(new NamedGraph("simplex-both-varfreq", freq(simplex(), add(c(0.125F), perlin()), add(c(-0.25F), simplex()))));

		graphs.add(new NamedGraph("add-posneg", add(simplex(), c(-2.0F))));
		graphs.add(new NamedGraph("add-signedzero", add(perlin(), c(-0.0F))));
		graphs.add(new NamedGraph("mul-neg-range", mul(simplex(), c(-1.75F))));
		graphs.add(new NamedGraph("mul-signedzero-lhs", mul(c(-0.0F), simplex())));
		graphs.add(new NamedGraph("mul-signedzero-rhs", mul(simplex(), c(-0.0F))));

		graphs.add(new NamedGraph("abs-simple", abs(add(simplex(), c(-1.0F)))));
		graphs.add(new NamedGraph("abs-nested", abs(abs(mul(perlin(), c(2.0F))))));
		graphs.add(new NamedGraph("invert-simple-nonzero", invert(add(simplex(), c(3.0F)))));
		graphs.add(new NamedGraph("invert-nested-nonzero", invert(invert(add(perlin(), c(2.0F))))));

		graphs.add(new NamedGraph("min-signedzero", min(c(0.0F), c(-0.0F))));
		graphs.add(new NamedGraph("min-signedzero-rev", min(c(-0.0F), c(0.0F))));
		graphs.add(new NamedGraph("max-signedzero", max(c(0.0F), c(-0.0F))));
		graphs.add(new NamedGraph("max-signedzero-rev", max(c(-0.0F), c(0.0F))));
		graphs.add(new NamedGraph("min-mixed", min(simplex(), c(-0.5F))));
		graphs.add(new NamedGraph("max-mixed", max(perlin(), c(-0.5F))));

		graphs.add(new NamedGraph("shiftseed-max", shift(perlin(), Integer.MAX_VALUE)));
		graphs.add(new NamedGraph("shiftseed-max-simplex", shift(simplex(), Integer.MAX_VALUE)));
		graphs.add(new NamedGraph("shiftseed-min", shift(simplex(), Integer.MIN_VALUE)));
		graphs.add(new NamedGraph("shiftseed-neg", shift(add(perlin(), c(1.0F)), -12345)));

		graphs.add(new NamedGraph("clamp-basic", clamp(simplex(), c(-1.0F), c(1.0F))));
		graphs.add(new NamedGraph("clamp-reversed", clamp(perlin(), c(1.0F), c(-1.0F))));
		graphs.add(new NamedGraph("clamp-negzero-bounds", clamp(simplex(), c(-0.0F), c(0.0F))));

		graphs.add(new NamedGraph("map-nonconst-alpha", map(add(simplex(), c(2.0F)), c(-1.0F), c(3.0F))));
		graphs.add(new NamedGraph("map-dynamic-from-to", map(add(perlin(), c(1.5F)), simplex(), mul(perlin(), c(-2.0F)))));

		graphs.add(new NamedGraph("nested-1", invert(add(abs(mul(simplex(), c(-2.0F))), c(1.0F)))));
		graphs.add(new NamedGraph("nested-2", clamp(
			add(freq(perlin(), c(0.5F), c(-1.5F)), abs(simplex())),
			c(-1.0F),
			mul(c(2.0F), c(1.5F))
		)));
		graphs.add(new NamedGraph("nested-3", map(
			add(abs(perlin()), c(1.0F)),
			min(simplex(), c(-0.25F)),
			max(shift(simplex(), 777), c(0.25F))
		)));
		graphs.add(new NamedGraph("nested-4", invert(invert(clamp(
			mul(add(perlin(), c(-1.0F)), abs(simplex())),
			c(-2.0F),
			c(3.0F)
		)))));
		graphs.add(new NamedGraph("nested-5", max(
			min(invert(add(simplex(), c(2.0F))), clamp(perlin(), c(-1.0F), c(0.5F))),
			shift(mul(abs(perlin()), c(-1.0F)), -42)
		)));

		return graphs;
	}
}
