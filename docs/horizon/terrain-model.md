# Terrain categories and registry model

The native kernel now has an owned terrain registry containing the original
18 built-ins, named inheritance and composite construction. Two fresh genuine
Java processes agree on 695 records; native debug/release match all 10,351 fields.
Checks cover every built-in category/flag/erosion modifier/render hue, aliases,
case-insensitive collisions, NONE/null parents, all 529 ordered pairs of the
23 initial/builtin/alias inputs, nested composites, ID lookup and category ordering.

The source CompositeTerrain.getDominant reads a.getCategory twice, so its sorted
lower-ID parent wins the other flags/category. Flatness still requires both
parents; erosion uses the smaller modifier. The port preserves and explicitly
checks this inherited behavior. A preexisting alias that shadows a future
composite name also wins, including its inherited properties. Category dominance
itself remains the independent ordinal-based method.

Native IDs belong to the owning kernel registry rather than a mutable global
registry shared across worlds. Initialization/registration order must remain
deterministic when the complete terrain provider is built. This finite corpus
does not qualify arbitrary prior-world registration history, Java subclass
extensions or the terrain generator. Native new names are explicitly limited to
nonempty ASCII strings up to 256 bytes, registry size to 65,536, and parent IDs
must belong to the registry. Rejection guards are observed by the probe.

This model supplies the properties needed by future continent, river and tile
filters; it does not produce a height, biome, tile or chunk. Use
`verify_native_noise.py --family terrain --report <new-report.json>`.
