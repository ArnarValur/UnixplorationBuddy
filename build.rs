use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CanonnVariant {
    name: Option<String>,
    reward: Option<u64>,
    #[serde(rename = "atmosphereType", default)]
    atmosphere_type: Vec<Option<String>>,
    #[serde(default)]
    bodies: Vec<Option<String>>,
    #[serde(rename = "primaryStars", default)]
    primary_stars: Vec<Option<String>>,
    #[serde(default)]
    ming: Option<f64>,
    #[serde(default)]
    maxg: Option<f64>,
    #[serde(default)]
    mint: Option<f64>,
    #[serde(default)]
    maxt: Option<f64>,
    #[serde(default)]
    minp: Option<f64>,
    #[serde(default)]
    maxp: Option<f64>,
    #[serde(default)]
    volcanism: Vec<Option<String>>,
}

fn main() {
    // Only rerun if canonn-data changes
    println!("cargo:rerun-if-changed=data/canonn");

    let canonn_dir = Path::new("data/canonn");
    if !canonn_dir.exists() {
        return;
    }

    let mut generated_code = String::new();
    generated_code.push_str("//! Strongly-typed static Canonn dataset generated at build time.\n\n");
    generated_code.push_str("/// Environmental and payout boundary data for a specific biological species variant.\n");
    generated_code.push_str("#[derive(Debug, Clone, PartialEq)]\n");
    generated_code.push_str("pub struct SpeciesVariant {\n");
    generated_code.push_str("    pub name: &'static str,\n");
    generated_code.push_str("    pub genus: &'static str,\n");
    generated_code.push_str("    pub reward: u64,\n");
    generated_code.push_str("    pub atmosphere_types: &'static [&'static str],\n");
    generated_code.push_str("    pub bodies: &'static [&'static str],\n");
    generated_code.push_str("    pub primary_stars: &'static [&'static str],\n");
    generated_code.push_str("    pub min_g: f64,\n");
    generated_code.push_str("    pub max_g: f64,\n");
    generated_code.push_str("    pub min_t: f64,\n");
    generated_code.push_str("    pub max_t: f64,\n");
    generated_code.push_str("    pub min_p: f64,\n");
    generated_code.push_str("    pub max_p: f64,\n");
    generated_code.push_str("    pub volcanism: &'static [&'static str],\n");
    generated_code.push_str("}\n\n");

    generated_code.push_str("pub static DATASET: &[SpeciesVariant] = &[\n");

    if let Ok(entries) = fs::read_dir(canonn_dir) {
        // Collect and sort entries to ensure deterministic code generation order
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("json"))
            .collect();
        paths.sort();

        for path in paths {
            let filename = path.file_stem().unwrap().to_str().unwrap().to_string();
            if let Ok(file_content) = fs::read_to_string(&path) {
                match serde_json::from_str::<HashMap<String, CanonnVariant>>(&file_content) {
                    Ok(variants_map) => {
                        // Sort variants by name for deterministic generation order
                        let mut variants: Vec<_> = variants_map.values().collect();
                        // Filter out variants that don't have a valid name
                        variants.retain(|v| v.name.is_some());
                        variants.sort_by(|a, b| a.name.as_ref().unwrap().cmp(b.name.as_ref().unwrap()));

                        for variant in variants {
                            let name = variant.name.as_ref().unwrap();
                            let reward = variant.reward.unwrap_or(0);

                            // Filter out any None values in string arrays
                            let atmos_vec: Vec<String> = variant.atmosphere_type.iter().flatten().cloned().collect();
                            let bodies_vec: Vec<String> = variant.bodies.iter().flatten().cloned().collect();
                            let stars_vec: Vec<String> = variant.primary_stars.iter().flatten().cloned().collect();
                            let volc_vec: Vec<String> = variant.volcanism.iter().flatten().cloned().collect();

                            let atmos = format!("&[{}]", atmos_vec.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<String>>().join(", "));
                            let bodies = format!("&[{}]", bodies_vec.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<String>>().join(", "));
                            let stars = format!("&[{}]", stars_vec.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<String>>().join(", "));
                            let volc = format!("&[{}]", volc_vec.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<String>>().join(", "));

                            generated_code.push_str("    SpeciesVariant {\n");
                            generated_code.push_str(&format!("        name: \"{}\",\n", name.replace("\"", "\\\"")));
                            generated_code.push_str(&format!("        genus: \"{}\",\n", filename.replace("\"", "\\\"")));
                            generated_code.push_str(&format!("        reward: {},\n", reward));
                            generated_code.push_str(&format!("        atmosphere_types: {},\n", atmos));
                            generated_code.push_str(&format!("        bodies: {},\n", bodies));
                            generated_code.push_str(&format!("        primary_stars: {},\n", stars));
                            generated_code.push_str(&format!("        min_g: {:.6},\n", variant.ming.unwrap_or(0.0)));
                            generated_code.push_str(&format!("        max_g: {:.6},\n", variant.maxg.unwrap_or(100.0)));
                            generated_code.push_str(&format!("        min_t: {:.6},\n", variant.mint.unwrap_or(0.0)));
                            generated_code.push_str(&format!("        max_t: {:.6},\n", variant.maxt.unwrap_or(10000.0)));
                            generated_code.push_str(&format!("        min_p: {:.6},\n", variant.minp.unwrap_or(0.0)));
                            generated_code.push_str(&format!("        max_p: {:.6},\n", variant.maxp.unwrap_or(10000.0)));
                            generated_code.push_str(&format!("        volcanism: {},\n", volc));
                            generated_code.push_str("    },\n");
                        }
                    }
                    Err(e) => {
                        panic!("Failed to parse JSON file {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    generated_code.push_str("];\n");

    let out_dir = Path::new("src/model/biology");
    let _ = fs::create_dir_all(out_dir);
    let mut file = File::create(out_dir.join("dataset.rs")).unwrap();
    file.write_all(generated_code.as_bytes()).unwrap();
}
