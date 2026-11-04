/* Tests real 26.2 classes and production adapters; no game-class substitutes. */
import com.google.gson.Gson;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.Codec;
import com.mojang.serialization.JsonOps;
import java.util.ArrayList;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Map;
import java.nio.file.Path;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.levelgen.DensityFunction;
import net.minecraft.world.level.levelgen.DensityFunctions;
import raccoonman.reterraforged.world.worldgen.densityfunction.ClampToNearestUnit;
import raccoonman.reterraforged.world.worldgen.densityfunction.LinearSplineFunction;

public final class DensityProbe {
    private static int assertions;
    private static final List<String> scenarios = new ArrayList<>();

    private static void check(boolean condition, String message) {
        assertions++;
        if (!condition) throw new AssertionError(message);
    }

    private static void equal(double actual, double expected, String message) {
        check(Double.doubleToRawLongBits(actual) == Double.doubleToRawLongBits(expected),
              message + ": expected " + expected + ", got " + actual);
    }

    private static DensityFunction.FunctionContext context(int i) {
        return new DensityFunction.SinglePointContext(i % 2 == 0 ? -17 : 16, i - 8, -i - 1);
    }

    private static final DensityFunction.ContextProvider CONTEXTS = new DensityFunction.ContextProvider() {
        @Override public DensityFunction.FunctionContext forIndex(int i) { return context(i); }
        @Override public void fillAllDirectly(double[] array, DensityFunction function) {
            for (int i = 0; i < array.length; i++) array[i] = function.compute(forIndex(i));
        }
    };

    private static void arrayMatchesScalar(DensityFunction function, int count) {
        double[] values = new double[count];
        function.fillArray(values, CONTEXTS);
        for (int i = 0; i < count; i++) equal(values[i], function.compute(context(i)), "fillArray mismatch at " + i);
        function.fillArray(new double[0], CONTEXTS);
    }

    private static <T extends DensityFunction> JsonElement roundTrip(Codec<T> codec, T original) {
        JsonElement json = codec.encodeStart(JsonOps.INSTANCE, original).getOrThrow();
        T decoded = codec.parse(JsonOps.INSTANCE, json).getOrThrow();
        JsonElement reencoded = codec.encodeStart(JsonOps.INSTANCE, decoded).getOrThrow();
        check(json.equals(reencoded), "JSON changed after round trip");
        equal(decoded.minValue(), original.minValue(), "Roundtrip minValue");
        equal(decoded.maxValue(), original.maxValue(), "Roundtrip maxValue");
        for (int i = 0; i < 17; i++) equal(decoded.compute(context(i)), original.compute(context(i)), "Roundtrip compute");
        arrayMatchesScalar(decoded, 17);
        return json;
    }

    private static void roundingAndBounds() {
        DensityFunction gradient = DensityFunctions.yClampedGradient(-4, 4, -1, 1);
        ClampToNearestUnit clamped = new ClampToNearestUnit(gradient, 4);
        double[] expected = {-.75, -.75, -.75, -.75, -.75, -.5, -.25, 0, .25, .5, .75, 1, 1.25, 1.25, 1.25, 1.25, 1.25};
        for (int i = 0; i < expected.length; i++) equal(clamped.compute(context(i)), expected[i], "Inherited rounding at y=" + (i - 8));
        equal(clamped.minValue(), -.75, "Clamp lower bound arithmetic");
        equal(clamped.maxValue(), 1.25, "Clamp upper bound arithmetic");
        arrayMatchesScalar(clamped, 17);
        check(new ClampToNearestUnit(DensityFunctions.constant(0), 0).compute(context(0)) == Double.POSITIVE_INFINITY,
              "Zero-resolution inherited behavior changed silently");
        scenarios.add("clamp_negative_positive_boundary_arithmetic_and_bounds");

        LinearSplineFunction spline = LinearSplineFunction.builder(gradient)
            .addPoint(-.5, -2).addPoint(0, 3).addPoint(.5, 1).build();
        double[] splineExpected = {-2, -2, -2, -2, -2, -2, -2, .5, 3, 2, 1, 1, 1, 1, 1, 1, 1};
        for (int i = 0; i < splineExpected.length; i++) equal(spline.compute(context(i)), splineExpected[i], "Spline boundary interpolation");
        equal(spline.minValue(), -2, "Spline min bound");
        equal(spline.maxValue(), 3, "Spline max bound");
        arrayMatchesScalar(spline, 17);

        double low = .1000000000001, high = .9000000000001;
        LinearSplineFunction floatBounds = LinearSplineFunction.builder(DensityFunctions.constant(0))
            .addPoint(-1, low).addPoint(1, high).build();
        equal(floatBounds.minValue(), (double) (float) low, "Inherited float min-bound rounding");
        equal(floatBounds.maxValue(), (double) (float) high, "Inherited float max-bound rounding");
        LinearSplineFunction single = LinearSplineFunction.builder(gradient).addPoint(0, -7).build();
        arrayMatchesScalar(single, 17);
        for (int i = 0; i < 17; i++) equal(single.compute(context(i)), -7, "Single-point spline");
        scenarios.add("spline_interpolation_endpoints_single_point_and_float_bounds");
    }

    private static void traversal() {
        DensityFunction zero = DensityFunctions.constant(0);
        DensityFunction ten = DensityFunctions.constant(10);
        DensityFunction twenty = DensityFunctions.constant(20);
        ClampToNearestUnit input = new ClampToNearestUnit(zero, 4);
        ClampToNearestUnit first = new ClampToNearestUnit(ten, 4);
        ClampToNearestUnit secondInner = new ClampToNearestUnit(twenty, 4);
        ClampToNearestUnit second = new ClampToNearestUnit(secondInner, 4);
        LinearSplineFunction root = LinearSplineFunction.builder(input).addPoint(-1, first).addPoint(1, second).build();

        List<DensityFunction> direct = new ArrayList<>();
        root.mapChildren(node -> { direct.add(node); return node; });
        check(direct.size() == 3 && direct.get(0) == input && direct.get(1) == first && direct.get(2) == second,
              "mapChildren must visit only the three immediate children, not self or grandchildren");
        List<DensityFunction> clampDirect = new ArrayList<>();
        second.mapChildren(node -> { clampDirect.add(node); return node; });
        check(clampDirect.size() == 1 && clampDirect.get(0) == secondInner, "Clamp mapChildren depth");

        Map<DensityFunction, Integer> leafVisits = new IdentityHashMap<>();
        leafVisits.put(zero, 0); leafVisits.put(ten, 0); leafVisits.put(twenty, 0);
        int[] wrapperVisits = {0, 0};
        LinearSplineFunction mapped = (LinearSplineFunction) root.mapAll(node -> {
            if (leafVisits.containsKey(node)) {
                leafVisits.put(node, leafVisits.get(node) + 1);
                return DensityFunctions.constant(node.compute(context(8)) + 1);
            }
            if (node instanceof ClampToNearestUnit) wrapperVisits[0]++;
            else if (node instanceof LinearSplineFunction) wrapperVisits[1]++;
            else throw new AssertionError("Unexpected traversal node " + node.getClass().getName());
            return node;
        });
        check(leafVisits.values().stream().allMatch(n -> n == 1), "Each leaf must be visited once, including nested spline values");
        check(wrapperVisits[0] == 4 && wrapperVisits[1] == 1, "Each wrapper must be visited exactly once");
        equal(((ClampToNearestUnit) mapped.input()).function().compute(context(8)), 1, "Input leaf was mapped");
        ClampToNearestUnit mappedFirst = (ClampToNearestUnit) mapped.points().get(0).getSecond();
        ClampToNearestUnit mappedSecond = (ClampToNearestUnit) mapped.points().get(1).getSecond();
        equal(mappedFirst.function().compute(context(8)), 11, "First value leaf was mapped");
        equal(((ClampToNearestUnit) mappedSecond.function()).function().compute(context(8)), 21,
              "Nested value leaf must be reached recursively");
        equal(root.compute(context(8)), 16.65625, "Original nested tree arithmetic");
        equal(mapped.compute(context(8)), 21.5, "Transformed nested tree arithmetic");
        equal(mapped.minValue(), 11.25, "Mapped tree recalculates min bound");
        equal(mapped.maxValue(), 21.5, "Mapped tree recalculates max bound");
        arrayMatchesScalar(mapped, 17);
        scenarios.add("mapChildren_direct_only_and_default_mapAll_exactly_once_nested_replacement");
    }

    public static void main(String[] args) throws Exception {
        SharedConstants.tryDetectVersion();
        check(SharedConstants.getCurrentVersion().id().equals("26.2"), "Classpath must contain Minecraft 26.2");
        Bootstrap.bootStrap();
        roundingAndBounds();
        traversal();
        JsonObject encoded = new JsonObject();
        encoded.add("clamp", roundTrip(ClampToNearestUnit.CODEC.codec(),
            new ClampToNearestUnit(DensityFunctions.yClampedGradient(-4, 4, -1, 1), 4)));
        scenarios.add("clamp_actual_26_2_JSON_codec_roundtrip_with_vanilla_gradient");
        encoded.add("spline", roundTrip(LinearSplineFunction.CODEC.codec(),
            LinearSplineFunction.builder(DensityFunctions.yClampedGradient(-4, 4, -1, 1))
                .addPoint(-1, DensityFunctions.yClampedGradient(-8, 8, -2, 2))
                .addPoint(0, 3).addPoint(1, -1).build()));
        scenarios.add("spline_actual_26_2_JSON_codec_roundtrip_with_vanilla_children");
        JsonElement explicitConfig = JsonParser.parseString("""
            {"input":{"type":"minecraft:y_clamped_gradient","from_y":-4,"to_y":4,"from_value":-1,"to_value":1},
             "points":[{"location":-1.5,"value":-2.25},{"location":0.5,"value":4.125}]}
            """);
        LinearSplineFunction parsed = LinearSplineFunction.CODEC.codec().parse(JsonOps.INSTANCE, explicitConfig).getOrThrow();
        JsonElement persisted = LinearSplineFunction.CODEC.codec().encodeStart(JsonOps.INSTANCE, parsed).getOrThrow();
        LinearSplineFunction reloaded = LinearSplineFunction.CODEC.codec().parse(JsonOps.INSTANCE, persisted).getOrThrow();
        equal(reloaded.points().get(0).getFirst(), -1.5, "Persisted negative point location");
        equal(reloaded.points().get(1).getFirst(), .5, "Persisted positive point location");
        equal(reloaded.points().get(0).getSecond().compute(context(8)), -2.25, "Persisted point value");
        equal(reloaded.points().get(1).getSecond().compute(context(8)), 4.125, "Persisted point value");
        for (int i = 0; i < 17; i++) equal(reloaded.compute(context(i)), parsed.compute(context(i)), "Parse/persist/reload compute");
        scenarios.add("named_point_configuration_parse_encode_parse_preserves_settings");
        check(LinearSplineFunction.CODEC.codec().parse(JsonOps.INSTANCE,
              JsonParser.parseString("{\"input\":0,\"points\":[]}")).error().isPresent(), "Empty spline must be rejected");
        check(LinearSplineFunction.CODEC.codec().parse(JsonOps.INSTANCE,
              JsonParser.parseString("{\"input\":0,\"points\":[[-1,2],[1,3]]}")).error().isPresent(), "Unsupported untyped point arrays must fail explicitly");
        JsonObject report = new JsonObject();
        report.addProperty("status", "pass");
        report.addProperty("assertions", assertions);
        report.addProperty("minecraft", SharedConstants.getCurrentVersion().id());
        report.addProperty("game_class_artifact", Path.of(DensityFunction.class.getProtectionDomain().getCodeSource().getLocation().toURI()).getFileName().toString());
        report.add("scenarios", new Gson().toJsonTree(scenarios));
        report.add("encoded_examples", encoded);
        report.addProperty("game_bootstrap", "real Minecraft Bootstrap.bootStrap completed");
        report.addProperty("scope", "Two production density adapters against resolved Minecraft 26.2 classes; not full mod or terrain");
        Bootstrap.realStdoutPrintln("DENSITY_QA_JSON=" + new Gson().toJson(report));
        Bootstrap.shutdownStdout();
    }
}
