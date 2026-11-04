# Native registry adapters

This separate crate translates bounded TerraForged registry JSON into the typed
native noise graphs. It is not activated in Horizon and cannot generate a chunk.
See [registry evidence and limits](../../docs/horizon/noise-registry.md).

It now also translates the five kernel preset groups and drives independent
debug/release comparisons for terrain, climate, river networks and complete
filtered tiles. See the [current checkpoint](../../docs/horizon/PROGRESS.md).
The probes receive configuration and coordinate/layout inputs, never expected
Java cells or prepared chunk snapshots.

The dependency-free arithmetic core remains separate. This adapter uses the
serde_json version already present in the inspected Horizon lockfile, with exact
decimal lexemes retained for f32 conversion. All dependencies are Cargo-locked.

Original TerraForged/ReTerraForged MIT attribution remains in the repository.
No game files or Mojang implementation code are included.
