"""Build the shared runtime probes with a NeoForge-only test entrypoint/provenance adapter."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[2]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--java-home", required=True, type=Path)
    parser.add_argument("--production-jar", required=True, type=Path)
    parser.add_argument("--output", type=Path, default=ROOT / "build/qa/neoforge-runtime/terraforged-neoforge-runtime-qa.jar")
    args = parser.parse_args()
    source = ROOT / "tests/runtime/terraforged/qa/RuntimeQaMod.java"
    original = source.read_text(encoding="utf-8")
    text = original.replace("import net.fabricmc.api.ModInitializer;", "import net.neoforged.fml.common.Mod;")
    text = text.replace("import net.fabricmc.fabric.api.event.lifecycle.v1.ServerLifecycleEvents;",
        "import net.neoforged.neoforge.common.NeoForge;\nimport net.neoforged.neoforge.event.server.ServerStartedEvent;")
    text = text.replace("import net.fabricmc.loader.api.FabricLoader;",
        "import net.neoforged.fml.ModList;\nimport net.neoforged.fml.loading.FMLLoader;")
    old_entry = """public final class RuntimeQaMod implements ModInitializer {"""
    assert text.count(old_entry) == 1
    text = text.replace(old_entry, '@Mod("terraforged_runtime_qa")\npublic final class RuntimeQaMod {')
    old_init = """    @Override
    public void onInitialize() {
        ServerLifecycleEvents.SERVER_STARTED.register(RuntimeQaMod::run);
    }"""
    assert text.count(old_init) == 1
    text = text.replace(old_init, """    public RuntimeQaMod() {
        NeoForge.EVENT_BUS.addListener((ServerStartedEvent event) -> run(event.getServer()));
    }""")
    text = text.replace("FabricLoader loader = FabricLoader.getInstance();", "FMLLoader loader = FMLLoader.getCurrent();")
    text = text.replace('require(!loader.isDevelopmentEnvironment(), "QA requires a production Fabric launch, not development mode");',
        'require(loader.isProduction(), "QA requires a production NeoForge launch, not development mode");')
    text = text.replace('var mod = loader.getModContainer("reterraforged").orElseThrow();',
        'var mod = ModList.get().getModContainerById("reterraforged").orElseThrow();')
    text = text.replace('loader.isDevelopmentEnvironment()', '!loader.isProduction()')
    text = text.replace('loader.getModContainer("fabricloader").orElseThrow().getMetadata().getVersion().getFriendlyString()',
        'ModList.get().getModContainerById("neoforge").orElseThrow().getModInfo().getVersion().toString()')
    text = text.replace('mod.getMetadata().getVersion().getFriendlyString()', 'mod.getModInfo().getVersion().toString()')
    text = text.replace('mod.getOrigin().getPaths().stream().map(Path::toString).toList()', 'List.of(origin.toString())')
    text = text.replace('import raccoonman.reterraforged.world.worldgen.feature.template.TemplateBiomeDecoratorProbe;',
        'import terraforged.qa.TemplateBiomeDecoratorProbe;')
    if "fabricmc" in text or "FabricLoader" in text:
        raise ValueError("Unexpected Fabric-specific probe code remains after the bounded adapter")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix="compile-", dir=args.output.parent))
    generated = work / "source/terraforged/qa/RuntimeQaMod.java"
    generated.parent.mkdir(parents=True)
    generated.write_text(text, encoding="utf-8")
    classpath = (ROOT / "neoforge/build/qa/target/compile-classpath.txt").read_text().strip()
    javac = args.java_home.resolve() / "bin/javac.exe"
    classes = work / "classes"
    classes.mkdir()
    terrain_probe_source = ROOT / "tests/terrain/TemplateBiomeDecoratorProbe.java"
    probe_text = terrain_probe_source.read_text().replace(
        "package raccoonman.reterraforged.world.worldgen.feature.template;",
        "package terraforged.qa;\nimport raccoonman.reterraforged.world.worldgen.feature.template.TemplateFeature;")
    probe_text = probe_text.replace("TemplateFeature.decoratorsForBiome(config,", "decoratorsForBiome(config,")
    # Named Java modules reject split packages. Keep the probe in its own test
    # module and invoke the same package-private production helper reflectively.
    probe_text = probe_text.replace("    public static void main(String[] args) throws Exception {", """    private static Object decoratorsForBiome(DecoratorConfig<?> config, Holder<Biome> biome) throws Exception {
        var method = TemplateFeature.class.getDeclaredMethod("decoratorsForBiome", DecoratorConfig.class, Holder.class);
        if (!method.trySetAccessible()) throw new IllegalStateException("Production helper is not accessible to the QA module");
        return method.invoke(null, config, biome);
    }

    public static void main(String[] args) throws Exception {""")
    terrain_probe = generated.parent / "TemplateBiomeDecoratorProbe.java"
    terrain_probe.write_text(probe_text, encoding="utf-8")
    result = subprocess.run([str(javac), "--release", "25", "-proc:none", "-encoding", "UTF-8",
        "-cp", str(args.production_jar.resolve()) + os.pathsep + classpath, "-d", str(classes),
        str(generated), str(terrain_probe)], capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=90)
    (work / "compile.log").write_text(result.stdout + result.stderr, encoding="utf-8")
    if result.returncode:
        print(result.stdout + result.stderr)
        return result.returncode
    metadata = '''license="MIT"
[[mods]]
modId="terraforged_runtime_qa"
version="0.1.0"
displayName="TerraForged NeoForge runtime QA companion"
description="Test-only artifact; never part of the production mod."
[[dependencies.terraforged_runtime_qa]]
modId="reterraforged"
type="required"
versionRange="[0,)"
ordering="AFTER"
side="BOTH"
'''
    with zipfile.ZipFile(args.output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("META-INF/neoforge.mods.toml", metadata)
        archive.writestr("META-INF/MANIFEST.MF", "Manifest-Version: 1.0\nImplementation-Title: Test-only TerraForged runtime QA\n\n")
        for file in sorted(classes.rglob("*.class")):
            archive.write(file, file.relative_to(classes).as_posix())
    report = {"status": "built", "compileExit": result.returncode, "output": str(args.output),
        "sha256": hashlib.sha256(args.output.read_bytes()).hexdigest(),
        "sharedProbeSourceSha256Lf": hashlib.sha256(original.replace("\r\n", "\n").encode()).hexdigest(),
        "generatedAdapterSha256Lf": hashlib.sha256(text.replace("\r\n", "\n").encode()).hexdigest(),
        "productionJarSha256": hashlib.sha256(args.production_jar.read_bytes()).hexdigest(),
        "adapterChanges": ["NeoForge ServerStartedEvent entrypoint", "NeoForge production/metadata provenance",
            "Relocate biome probe to test module and invoke same package-private helper reflectively to avoid split packages"],
        "notEvidenceOf": ["Server boot", "Preset export", "Custom generation"]}
    (args.output.parent / "build-result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
