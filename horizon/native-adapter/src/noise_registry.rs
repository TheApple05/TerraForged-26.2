//! Bounded adapter for source Noise/Domains/CurveFunction JSON, not Mojang code.
use serde_json::{Map, Value};
use std::{collections::BTreeMap, fmt};
use terraforged_native_core::{
    domain::Domain,
    erosion::{BlendMode, Erosion},
    graph::{CellFunction, CurveFunction, GraphError, Noise},
    line::Line,
    perlin::{Interpolation, Perlin},
    perlin2::Perlin2,
    ridges::{Billow, PerlinRidge, SimplexRidge},
    simplex::Simplex,
    simplex2::Simplex2,
    sources::{Cubic, White},
    worley::{DistanceFunction, EdgeFunction},
};

const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_DOCUMENTS: usize = 4096;
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 1_000_000;
#[derive(Debug, Clone)]
pub struct DecodeError(pub String);
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for DecodeError {}
type Result<T> = std::result::Result<T, DecodeError>;
impl From<GraphError> for DecodeError {
    fn from(error: GraphError) -> Self {
        Self(format!("Graph resource/parameter limit: {error:?}"))
    }
}
fn error(message: impl Into<String>) -> DecodeError {
    DecodeError(message.into())
}
fn json(text: &str) -> Result<Value> {
    if text.len() > MAX_BYTES {
        return Err(error("Registry JSON exceeds 16 MiB"));
    }
    // serde_json canonicalizes integer -0 to unsigned zero, even with arbitrary
    // precision. Gson's Float parser preserves its sign. Normalize this one
    // numeric token outside strings before deserialization; no data is omitted.
    let mut normalized = Vec::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut quoted = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            normalized.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            quoted = true;
        }
        if byte == b'-'
            && bytes.get(index + 1) == Some(&b'0')
            && bytes
                .get(index + 2)
                .is_none_or(|b| b.is_ascii_whitespace() || b",]}".contains(b))
        {
            normalized.extend_from_slice(b"-0.0");
            index += 2;
        } else {
            normalized.push(byte);
            index += 1;
        }
    }
    serde_json::from_slice(&normalized).map_err(|e| error(format!("Invalid registry JSON: {e}")))
}
fn identifier(text: &str) -> Result<String> {
    if text.len() > 256 {
        return Err(error("Identifier too long"));
    }
    let (namespace, path) = text.split_once(':').unwrap_or(("minecraft", text));
    if namespace.is_empty()
        || path.is_empty()
        || !namespace
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-".contains(&b))
        || !path
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-/".contains(&b))
    {
        return Err(error(format!("Invalid identifier {text:?}")));
    }
    Ok(format!("{namespace}:{path}"))
}
fn object(value: &Value) -> Result<&Map<String, Value>> {
    value.as_object().ok_or_else(|| error("Expected object"))
}
fn get<'a>(value: &'a Value, key: &str) -> Result<&'a Value> {
    object(value)?
        .get(key)
        .ok_or_else(|| error(format!("Missing field {key}")))
}
fn string(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(|| error("Expected string"))
}
fn field_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    string(get(value, key)?)
}
fn number(value: &Value) -> Result<f32> {
    let Value::Number(number) = value else {
        return Err(error("Expected float number"));
    };
    // Preserve decimal-to-float rounding instead of an implicit f64 double rounding.
    let f = number
        .to_string()
        .parse::<f32>()
        .map_err(|e| error(format!("Invalid float: {e}")))?;
    if !f.is_finite() {
        return Err(error("Only finite numeric parameters are supported"));
    }
    Ok(f)
}
fn float(value: &Value, key: &str) -> Result<f32> {
    number(get(value, key)?)
}
fn int(value: &Value, key: &str) -> Result<i32> {
    let n = get(value, key)?
        .as_i64()
        .ok_or_else(|| error(format!("Expected integral signed32 field {key}")))?;
    i32::try_from(n).map_err(|_| error(format!("Field {key} is outside signed32 range")))
}
fn interpolation(value: &str) -> Result<Interpolation> {
    match value {
        "LINEAR" => Ok(Interpolation::Linear),
        "CURVE3" => Ok(Interpolation::Curve3),
        "CURVE4" => Ok(Interpolation::Curve4),
        _ => Err(error(format!("Unknown interpolation {value}"))),
    }
}
fn distance(value: &str) -> Result<DistanceFunction> {
    match value {
        "EUCLIDEAN" => Ok(DistanceFunction::Euclidean),
        "MANHATTAN" => Ok(DistanceFunction::Manhattan),
        "NATURAL" => Ok(DistanceFunction::Natural),
        _ => Err(error(format!("Unknown distance function {value}"))),
    }
}
fn cell(value: &str) -> Result<CellFunction> {
    match value {
        "CELL_VALUE" => Ok(CellFunction::CellValue),
        "NOISE_LOOKUP" => Ok(CellFunction::NoiseLookup),
        "DISTANCE" => Ok(CellFunction::Distance),
        _ => Err(error(format!("Unknown cell function {value}"))),
    }
}
fn edge(value: &str) -> Result<EdgeFunction> {
    match value {
        "DISTANCE_2" => Ok(EdgeFunction::Distance2),
        "DISTANCE_2_ADD" => Ok(EdgeFunction::Distance2Add),
        "DISTANCE_2_SUB" => Ok(EdgeFunction::Distance2Sub),
        "DISTANCE_2_MUL" => Ok(EdgeFunction::Distance2Mul),
        "DISTANCE_2_DIV" => Ok(EdgeFunction::Distance2Div),
        _ => Err(error(format!("Unknown edge function {value}"))),
    }
}
fn source<T, E: fmt::Debug>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|e| error(format!("Unsupported source parameters: {e:?}")))
}

/// Fully resolved, bound registry. Construction publishes nothing on any failure.
#[derive(Debug)]
pub struct NoiseRegistry {
    noises: BTreeMap<String, Noise>,
}
impl NoiseRegistry {
    /// Values may be JSON documents or encoded JSON strings (the frozen oracle format).
    pub fn from_json(text: &str) -> Result<Self> {
        let input = json(text)?;
        let object = object(&input)?;
        if object.len() > MAX_DOCUMENTS {
            return Err(error("Registry exceeds 4096 definitions"));
        }
        let mut definitions = BTreeMap::new();
        for (id, value) in object {
            let id = identifier(id)?;
            let value = if let Some(text) = value.as_str() {
                json(text)?
            } else {
                value.clone()
            };
            if definitions.insert(id.clone(), value).is_some() {
                return Err(error(format!("Duplicate normalized identifier {id}")));
            }
        }
        let mut resolver = Resolver {
            definitions,
            ready: BTreeMap::new(),
            active: Vec::new(),
            nodes: 0,
        };
        for id in resolver.definitions.keys().cloned().collect::<Vec<_>>() {
            resolver.resolve(&id, 0)?;
        }
        Ok(Self {
            noises: resolver.ready,
        })
    }
    pub fn get(&self, id: &str) -> Result<Noise> {
        let id = identifier(id)?;
        self.noises
            .get(&id)
            .cloned()
            .ok_or_else(|| error(format!("Missing noise {id}")))
    }
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Noise)> {
        self.noises.iter().map(|(id, n)| (id.as_str(), n))
    }
    pub fn len(&self) -> usize {
        self.noises.len()
    }
    pub fn is_empty(&self) -> bool {
        self.noises.is_empty()
    }
}
struct Resolver {
    definitions: BTreeMap<String, Value>,
    ready: BTreeMap<String, Noise>,
    active: Vec<String>,
    nodes: usize,
}
impl Resolver {
    fn guard(&mut self, depth: usize) -> Result<()> {
        if depth > MAX_DEPTH {
            return Err(error("Decode nesting/reference depth exceeds 64"));
        }
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err(error("Decode node budget exceeded"));
        }
        Ok(())
    }
    fn resolve(&mut self, id: &str, depth: usize) -> Result<Noise> {
        self.guard(depth)?;
        if let Some(noise) = self.ready.get(id) {
            return Ok(noise.clone());
        }
        if self.active.iter().any(|n| n == id) {
            return Err(error(format!(
                "Cyclic noise reference: {} -> {id}",
                self.active.join(" -> ")
            )));
        }
        let document = self
            .definitions
            .get(id)
            .cloned()
            .ok_or_else(|| error(format!("Missing noise reference {id}")))?;
        self.active.push(id.to_owned());
        let decoded = self.noise(&document, depth + 1);
        self.active.pop();
        let decoded = decoded.map_err(|e| error(format!("In {id}: {e}")))?;
        self.ready.insert(id.to_owned(), decoded.clone());
        Ok(decoded)
    }
    fn child(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        let noise = if let Some(id) = value.as_str() {
            self.resolve(&identifier(id)?, depth + 1)?
        } else {
            self.noise(value, depth + 1)?
        };
        Ok(Noise::holder(noise)?)
    }
    fn noise(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        self.guard(depth)?;
        if value.is_number() {
            let n = number(value)?;
            if n.abs() > 1_000_000.0 {
                return Err(error("Literal noise value exceeds source +/-1000000 range"));
            }
            return Ok(Noise::constant(n)?);
        }
        let kind = identifier(field_string(value, "type")?)?;
        let decoder: fn(&mut Self, &Value, usize) -> Result<Noise> = match kind.as_str() {
            "reterraforged:constant" => Self::decode_constant,
            "reterraforged:white" => Self::decode_white,
            "reterraforged:perlin" => Self::decode_perlin,
            "reterraforged:perlin2" => Self::decode_perlin2,
            "reterraforged:simplex" => Self::decode_simplex,
            "reterraforged:simplex2" => Self::decode_simplex2,
            "reterraforged:perlin_ridge" => Self::decode_perlin_ridge,
            "reterraforged:simplex_ridge" => Self::decode_simplex_ridge,
            "reterraforged:billow" => Self::decode_billow,
            "reterraforged:cubic" => Self::decode_cubic,
            "reterraforged:sin" => Self::decode_sin,
            "reterraforged:worley" => Self::decode_worley,
            "reterraforged:worley_edge" => Self::decode_worley_edge,
            "reterraforged:shift" => Self::decode_shift,
            "reterraforged:frequency" => Self::decode_frequency,
            "reterraforged:add" => Self::decode_add,
            "reterraforged:multiply" => Self::decode_multiply,
            "reterraforged:power" => Self::decode_power,
            "reterraforged:power_curve" => Self::decode_power_curve,
            "reterraforged:curve" => Self::decode_curve,
            "reterraforged:gradient" => Self::decode_gradient,
            "reterraforged:terrace" => Self::decode_terrace,
            "reterraforged:advanced_terrace" => Self::decode_advanced_terrace,
            "reterraforged:invert" => Self::decode_invert,
            "reterraforged:blend" => Self::decode_blend,
            "reterraforged:alpha" => Self::decode_alpha,
            "reterraforged:boost" => Self::decode_boost,
            "reterraforged:steps" => Self::decode_steps,
            "reterraforged:abs" => Self::decode_abs,
            "reterraforged:map" => Self::decode_map,
            "reterraforged:clamp" => Self::decode_clamp,
            "reterraforged:threshold" => Self::decode_threshold,
            "reterraforged:min" => Self::decode_min,
            "reterraforged:max" => Self::decode_max,
            "reterraforged:warp" => Self::decode_warp,
            "reterraforged:erosion" => Self::decode_erosion,
            "reterraforged:linear_spline" => Self::decode_linear_spline,
            "reterraforged:cache" => Self::decode_cache,
            "reterraforged:legacy_temperature" => Self::decode_legacy_temperature,
            "reterraforged:legacy_moisture" => Self::decode_legacy_moisture,
            "reterraforged:line" => Self::decode_line,
            _ => return Err(error(format!("Unsupported noise type {kind}"))),
        };
        decoder(self, value, depth)
    }
    fn decode_constant(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok({
            let n = float(value, "value")?;
            if n.abs() > 1_000_000.0 {
                return Err(error("Constant exceeds source value range"));
            }
            Noise::constant(n)?
        })
    }
    fn decode_white(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::white(White::new(float(value, "frequency")?))?)
    }
    fn decode_perlin(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::perlin(source(Perlin::new(
            int(value, "seed")?,
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_perlin2(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::perlin2(source(Perlin2::new(
            int(value, "seed")?,
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_simplex(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::simplex(source(Simplex::new(
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_simplex2(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::simplex2(source(Simplex2::new(
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_perlin_ridge(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::perlin_ridge(source(PerlinRidge::new(
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_simplex_ridge(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::simplex_ridge(source(SimplexRidge::new(
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_billow(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::billow(source(Billow::new(
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
            interpolation(field_string(value, "interpolation")?)?,
        ))?)?)
    }
    fn decode_cubic(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::cubic(source(Cubic::new(
            float(value, "frequency")?,
            int(value, "octaves")?,
            float(value, "lacunarity")?,
            float(value, "gain")?,
        ))?)?)
    }
    fn decode_sin(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::sin(
            float(value, "frequency")?,
            self.child(get(value, "alpha")?, depth + 1)?,
        )?)
    }
    fn decode_worley(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::worley(
            float(value, "frequency")?,
            float(value, "distance")?,
            cell(field_string(value, "cell_function")?)?,
            distance(field_string(value, "distance_function")?)?,
            self.child(get(value, "lookup")?, depth + 1)?,
        )?)
    }
    fn decode_worley_edge(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::worley_edge(
            float(value, "frequency")?,
            float(value, "distance")?,
            edge(field_string(value, "edge_function")?)?,
            distance(field_string(value, "distance_function")?)?,
        )?)
    }
    fn decode_shift(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::shift_seed(
            self.child(get(value, "input")?, depth + 1)?,
            int(value, "shift")?,
        )?)
    }
    fn decode_frequency(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::frequency(
            self.child(get(value, "input")?, depth + 1)?,
            self.child(get(value, "x_freq")?, depth + 1)?,
            self.child(get(value, "z_freq")?, depth + 1)?,
        )?)
    }
    fn decode_add(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::add(
            self.child(get(value, "input1")?, depth + 1)?,
            self.child(get(value, "input2")?, depth + 1)?,
        )?)
    }
    fn decode_multiply(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::multiply(
            self.child(get(value, "input1")?, depth + 1)?,
            self.child(get(value, "input2")?, depth + 1)?,
        )?)
    }
    fn decode_power(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::power(
            self.child(get(value, "input")?, depth + 1)?,
            float(value, "power")?,
        )?)
    }
    fn decode_power_curve(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::power_curve(
            self.child(get(value, "input")?, depth + 1)?,
            float(value, "power")?,
        )?)
    }
    fn decode_curve(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::curve(
            self.child(get(value, "input")?, depth + 1)?,
            self.curve(get(value, "curve_function")?, depth + 1)?,
        )?)
    }
    fn decode_gradient(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok({
            Noise::gradient(
                self.child(get(value, "input")?, depth + 1)?,
                self.child(get(value, "lower")?, depth + 1)?,
                self.child(get(value, "upper")?, depth + 1)?,
                self.child(get(value, "strength")?, depth + 1)?,
            )?
        })
    }
    fn decode_terrace(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::terrace(
            self.child(get(value, "input")?, depth + 1)?,
            self.child(get(value, "ramp")?, depth + 1)?,
            self.child(get(value, "cliff")?, depth + 1)?,
            self.child(get(value, "ramp_height")?, depth + 1)?,
            float(value, "blend_range")?,
            int(value, "steps")?,
        )?)
    }
    fn decode_advanced_terrace(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::advanced_terrace(
            self.child(get(value, "source")?, depth + 1)?,
            self.child(get(value, "modulation")?, depth + 1)?,
            self.child(get(value, "mask")?, depth + 1)?,
            self.child(get(value, "slope")?, depth + 1)?,
            float(value, "blend_min")?,
            float(value, "blend_max")?,
            int(value, "steps")?,
            int(value, "octaves")?,
        )?)
    }
    fn decode_invert(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::invert(self.child(get(value, "input")?, depth + 1)?)?)
    }
    fn decode_blend(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::blend(
            self.child(get(value, "alpha")?, depth + 1)?,
            self.child(get(value, "lower")?, depth + 1)?,
            self.child(get(value, "upper")?, depth + 1)?,
            float(value, "mid")?,
            float(value, "range")?,
            interpolation(field_string(value, "interpolation")?)?,
        )?)
    }
    fn decode_alpha(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::alpha(
            self.child(get(value, "input")?, depth + 1)?,
            self.child(get(value, "alpha")?, depth + 1)?,
        )?)
    }
    fn decode_boost(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::boost(
            self.child(get(value, "input")?, depth + 1)?,
            int(value, "iterations")?,
        )?)
    }
    fn decode_steps(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::steps(
            self.child(get(value, "input")?, depth + 1)?,
            self.child(get(value, "steps")?, depth + 1)?,
            self.child(get(value, "slope_min")?, depth + 1)?,
            self.child(get(value, "slope_max")?, depth + 1)?,
            self.curve(get(value, "slope_curve")?, depth + 1)?,
        )?)
    }
    fn decode_abs(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::abs(self.child(get(value, "input")?, depth + 1)?)?)
    }
    fn decode_map(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::map(
            self.child(get(value, "alpha")?, depth + 1)?,
            self.child(get(value, "from")?, depth + 1)?,
            self.child(get(value, "to")?, depth + 1)?,
        )?)
    }
    fn decode_clamp(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::clamp(
            self.child(get(value, "input")?, depth + 1)?,
            self.child(get(value, "min")?, depth + 1)?,
            self.child(get(value, "max")?, depth + 1)?,
        )?)
    }
    fn decode_threshold(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok({
            Noise::threshold(
                self.child(get(value, "input")?, depth + 1)?,
                self.child(get(value, "lower")?, depth + 1)?,
                self.child(get(value, "upper")?, depth + 1)?,
                self.child(get(value, "threshold")?, depth + 1)?,
            )?
        })
    }
    fn decode_min(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::minimum(
            self.child(get(value, "input1")?, depth + 1)?,
            self.child(get(value, "input2")?, depth + 1)?,
        )?)
    }
    fn decode_max(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::maximum(
            self.child(get(value, "input1")?, depth + 1)?,
            self.child(get(value, "input2")?, depth + 1)?,
        )?)
    }
    fn decode_warp(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok({
            Noise::warp(
                self.child(get(value, "input")?, depth + 1)?,
                self.domain(get(value, "domain")?, depth + 1)?,
            )?
        })
    }
    fn decode_erosion(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok({
            let blend = match field_string(value, "blend_mode")? {
                "constant" => BlendMode::Constant,
                "input_linear" => BlendMode::InputLinear,
                "output_linear" => BlendMode::OutputLinear,
                other => return Err(error(format!("Unknown erosion blend {other}"))),
            };
            Noise::erosion(
                self.child(get(value, "input")?, depth + 1)?,
                source(Erosion::new(
                    int(value, "seed")?,
                    int(value, "octaves")?,
                    float(value, "strength")?,
                    float(value, "grid_size")?,
                    float(value, "amplitude")?,
                    float(value, "lacunarity")?,
                    float(value, "distance_falloff")?,
                    blend,
                ))?,
            )?
        })
    }
    fn decode_linear_spline(&mut self, _value: &Value, _depth: usize) -> Result<Noise> {
        Err(error(
            "Source LinearSpline JSON pair codec is unusable; use the typed numerical API, no invented wire fallback",
        ))
    }
    fn decode_cache(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::cache(self.child(get(value, "noise")?, depth + 1)?)?)
    }
    fn decode_legacy_temperature(&mut self, value: &Value, _depth: usize) -> Result<Noise> {
        Ok(Noise::legacy_temperature(
            float(value, "frequency")?,
            int(value, "power")?,
        )?)
    }
    fn decode_legacy_moisture(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::legacy_moisture(
            self.child(get(value, "source")?, depth + 1)?,
            int(value, "power")?,
        )?)
    }
    // The source codec repeats radiusSq for all three fields. Preserve that wire contract.
    fn decode_line(&mut self, value: &Value, depth: usize) -> Result<Noise> {
        Ok(Noise::line(
            Line::new(
                float(value, "x1")?,
                float(value, "z1")?,
                float(value, "x2")?,
                float(value, "z2")?,
                float(value, "feather")?,
            ),
            self.child(get(value, "radiusSq")?, depth + 1)?,
            self.child(get(value, "radiusSq")?, depth + 1)?,
            self.child(get(value, "radiusSq")?, depth + 1)?,
        )?)
    }
    fn domain(&mut self, value: &Value, depth: usize) -> Result<Domain> {
        self.guard(depth)?;
        let kind = identifier(field_string(value, "type")?)?;
        Ok(match kind.as_str() {
            "reterraforged:direct" => Domain::direct()?,
            "reterraforged:domain" => Domain::warp(
                self.child(get(value, "x")?, depth + 1)?,
                self.child(get(value, "z")?, depth + 1)?,
                self.child(get(value, "distance")?, depth + 1)?,
            )?,
            "reterraforged:direction" => Domain::direction(
                self.child(get(value, "direction")?, depth + 1)?,
                self.child(get(value, "strength")?, depth + 1)?,
            )?,
            "reterraforged:add" => Domain::add(
                self.domain(get(value, "input1")?, depth + 1)?,
                self.domain(get(value, "input2")?, depth + 1)?,
            )?,
            "reterraforged:compound" => Domain::compound(
                self.domain(get(value, "input1")?, depth + 1)?,
                self.domain(get(value, "input2")?, depth + 1)?,
            )?,
            _ => return Err(error(format!("Unsupported domain type {kind}"))),
        })
    }
    fn curve(&mut self, value: &Value, depth: usize) -> Result<CurveFunction> {
        self.guard(depth)?;
        let kind = identifier(field_string(value, "type")?)?;
        match kind.as_str() {
            "reterraforged:interpolation" => Ok(CurveFunction::Interpolation(interpolation(
                field_string(value, "value")?,
            )?)),
            "reterraforged:scurve" => Ok(CurveFunction::scurve(
                float(value, "lower")?,
                float(value, "upper")?,
            )),
            _ => Err(error(format!("Unsupported curve function {kind}"))),
        }
    }
}
