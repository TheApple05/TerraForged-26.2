# Bounded native registry adapter

The separate native-adapter crate resolves literal noises, all source noise type
IDs, five domain types, two curve functions and named references. It builds the
complete bound registry before publishing it. Missing/cyclic references, unknown
types/enums, absent fields and unsupported numeric/graph limits return explicit
errors; no default terrain or noise substitutes for a rejected document.

Two genuine production game JVMs loaded the same qualified preset datapack and
captured its actual 26-entry registry, plus 52 synthetic codec cases. The final
corpus contains 75 successful roots and three expected source rejections, with
130,128 records / 130,278 fields per native profile. Debug/release both match.
This tests decoding and numerical observations, not the cell/river/tile kernel.

The initial native run found two adapter defects. A large recursive match frame
overflowed the ordinary Windows debug stack. Per-type decoder functions remove
that frame multiplication without increasing the process/thread stack. serde_json
also canonicalized the integer token -0 to positive zero. The adapter preserves
that numeric token outside quoted strings before parsing, and parses decimal
float lexemes directly to f32 to avoid double rounding. The corpus includes
signed zero, decimal half-way boundaries and subnormals. Direct/encoded negative
zero and quoted identifier references also have explicit checks. Failed native
reports v1/v2 and the passing v3/v4 reports remain separate and immutable.

The upstream registry contains two unused terrain/erosion and terrain/ridges
definitions requesting 200 Perlin octaves. They produce 3,420 nonfinite sample
records in this matrix. Native Perlin/Perlin2 now admit 0..256 octaves and match
both those classifications and finite 256-octave inputs. Other source limits
are unchanged. These observations do not make the nonfinite definitions valid
terrain outputs; the default cell kernel uses terrain/ground instead. No Java
release artifact was changed to hide the inherited behavior.

The source noise LinearSpline JSON codec is unusable: Codec.pair decodes its
float first and then passes the consumed/empty remainder to the noise codec.
The actual target DataFixerUpper10.0.21 bytecode confirms that contract
(dependency JAR SHA256 3547cb1da8993fb7ebd42494d0e78997d2cfbe90c421f5efd022863e6909941e).
Two initial game runs rejected the assumed [position,noise] format. Corrected
captures explicitly confirm rejection of pair-array, numeric and empty point
lists. The adapter rejects this legacy wire type and retains the already verified
typed LinearSpline API. It does not invent a compatible JSON schema. Line's
separate legacy codec repeats radiusSq for radius/fade-in/fade-out; that wire
contract is preserved and checked. Neither unused wire limitation changes the
published Java dev23 JARs.

The adapter pins serde_json1.0.151, matching the inspected Horizon lockfile; Cargo
locks every transitive dependency. The core arithmetic crate remains dependency
free. Inputs are bounded to 16MiB, 4096 definitions, one million decoder nodes,
64 combined decoder/reference steps and the core graph limits. Integer parameters
must be actual signed32 integers and numeric parameters finite. Unsupported
parameter domains, arbitrary packs, complete preset interpretation, native
terrain/chunks, client/save behavior and launcher packaging remain unfinished.

Use `verify_native_registry.py --report <new-report.json>`. CI fetches only locked
Rust libraries, then runs all comparisons offline on Windows and Linux. The
adapter includes no Mojang implementation code, game assets or Java runtime.
