# Native world selector and real creation

The source integration now pins `TheApple05/horizon` commit
`61680d714bad3496a0ca5cef31537166f1232c4b` (fork PR7). Its bounded custom-preset
API adds metadata callbacks to the ordinary selector and saved-generator reader.
Registration retains no terrain kernel or world owner. Vanilla selector entries
keep their priority; the native entry cycles after them and wraps to Normal.

`stage_native_host.py --ui` includes the full source engine, automatic native
world selection and the two preset-registration hooks. The translated selector
label is **World Type: TerraForged**. Its actual settings writer creates the
native Overworld stem and ordinary Nether/End stems. The host API tests cover
rejected registrations, stem validation, selector cycling and recognition.

The [actual UI v2 run](../validation/horizon-native-preset-ui-v2.json) starts at
the title screen in a fresh isolated home, enters Create New World, types seed
8675309 and selects TerraForged. Structures remain enabled and the bonus chest
disabled. Commands are enabled through the UI solely for later QA teleports.
No world save or spawn position is supplied by the harness. The survival player
settles on solid ground with full health and food. Two fresh client processes
save and reopen that world, then check the original and a newly generated distant
FULL chunk. All 1,536 stored quart biomes in each tested chunk equal the actual
native factory and differ from the vanilla factory. Both client exits are clean.

The [first attempt](../validation/horizon-native-preset-ui-v1.json) exceeded a
30-second Create acknowledgement timeout and remains failed. The longer bound
allows that same production build to complete. A separate spawn diagnostic
shows the inherited vanilla search taking about 97 seconds on this machine
with competing QA work. This is not a controlled performance measurement.
The source TerraForged configured-center search is not yet installed by this
UI change; that remains a concrete fidelity and startup-cost follow-up.

The exact tested archive has SHA256
`532d1594a520530c0b44014e616376aab915c578afa6559ca1c7fa93a855c806`.
The [reader check](../validation/horizon-native-preset-package-v2.json) accepts
it with the actual native Bundle, Package and Content readers, and rejects the
historical malformed archive and a deliberately mismatched version. The v1
reader invocation used the accepted archive as its negative control, so its
failure is a harness-input error; v2 uses the original malformed archive.

Screenshot review accepts the translated selector. The v2 terrain views point
mostly at sky and do not establish terrain quality; they are not accepted as
scenery or visual-regression evidence. The runtime save and player-state checks
remain separate from that failed visual gate. The QA verifier is being improved
to wait for terrain and settle the camera before recording views.

All tests copy the player's existing prepared resources into isolated homes.
Manifest, resource index, facts and worldgen JSON bytes in both source and copy
are checked unchanged. No game files are downloaded. This is source-built client
and dev-mod evidence; standard launcher installation, all 17 native biome
resources/custom feature execution, original carver configuration, structure
coverage, controlled native performance and mod combinations remain open.

The Java 26.2 Fabric and NeoForge releases and their independent qualification
are unchanged by this native UI integration.
