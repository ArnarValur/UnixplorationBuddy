//! Galactic region lookup from StarPos coordinates.
//!
//! Uses a 2D RLE-encoded bitmap (2048×2048 pixels) to determine
//! which of the 42 galactic regions a star system belongs to.
//! Y coordinate is ignored — regions are defined on the galactic X-Z plane.
//!
//! Algorithm ported from ExploData's RegionMap.py.

mod region_data;

use region_data::{REGION_DATA, REGION_GROUPS, REGION_NAMES, ROW_OFFSETS};

// Galactic coordinate origin for the bitmap
const X0: f64 = -49985.0;
const Z0: f64 = -24105.0;

// Scale factor: 83 ly per 4096 pixels
const SCALE_NUM: f64 = 83.0;
const SCALE_DEN: f64 = 4096.0;

/// Result of a galactic region lookup.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionResult {
    /// Region ID (1-42). 0 means outside any region.
    pub id: u8,
    /// Region name (e.g. "Galactic Centre", "Inner Orion Spur").
    pub name: &'static str,
}

/// Determine the galactic region from StarPos coordinates.
///
/// Takes the 3D galactic coordinates [x, y, z] from FSDJump/Location events.
/// Y is ignored — regions are a 2D top-down projection.
///
/// Returns `Some(RegionResult)` if the coordinates fall within a known region,
/// or `None` if outside the mapped area or in empty space.
pub fn find_region(x: f64, _y: f64, z: f64) -> Option<RegionResult> {
    // Convert galactic coordinates to bitmap pixel coordinates
    let px = ((x - X0) * SCALE_NUM / SCALE_DEN) as i32;
    let pz = ((z - Z0) * SCALE_NUM / SCALE_DEN) as i32;

    // Bounds check
    if pz < 0 || pz >= ROW_OFFSETS.len() as i32 || px < 0 {
        return None;
    }

    let row_idx = pz as usize;
    let (start, count) = ROW_OFFSETS[row_idx];
    let start = start as usize;
    let count = count as usize;

    // Walk the RLE pairs for this row
    let mut rx: i32 = 0;
    let mut region_id: u8 = 0;

    for i in 0..count {
        let (run_length, rv) = REGION_DATA[start + i];
        if px < rx + run_length as i32 {
            region_id = rv;
            break;
        }
        rx += run_length as i32;
    }

    if region_id == 0 {
        return None;
    }

    Some(RegionResult {
        id: region_id,
        name: REGION_NAMES[region_id as usize],
    })
}

/// Check if a region ID belongs to a named bio region group.
///
/// Group names match BioScan's region_map keys (e.g. "orion-cygnus",
/// "sagittarius-carina", "perseus", "outer", "exterior", etc.)
pub fn region_in_group(region_id: u8, group_name: &str) -> bool {
    for &(name, ids) in REGION_GROUPS.iter() {
        if name == group_name {
            return ids.contains(&region_id);
        }
    }
    false
}

/// Get all group names that a region ID belongs to.
#[allow(dead_code)]
pub fn region_groups(region_id: u8) -> Vec<&'static str> {
    REGION_GROUPS
        .iter()
        .filter(|(_, ids)| ids.contains(&region_id))
        .map(|(name, _)| *name)
        .collect()
}

/// Get the region name for a given ID (1-42), or None for invalid/zero.
#[allow(dead_code)]
pub fn region_name(id: u8) -> Option<&'static str> {
    if id == 0 || id as usize >= REGION_NAMES.len() {
        return None;
    }
    Some(REGION_NAMES[id as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sol_is_inner_orion_spur() {
        // Sol is at approximately (0, 0, 0) in galactic coordinates
        let result = find_region(0.0, 0.0, 0.0);
        assert!(result.is_some(), "Sol should be in a known region");
        let r = result.unwrap();
        assert_eq!(r.id, 18, "Sol should be in Inner Orion Spur (ID 18)");
        assert_eq!(r.name, "Inner Orion Spur");
    }

    #[test]
    fn sagittarius_a_is_galactic_centre() {
        // Sagittarius A* is at approximately (25.21875, -20.90625, 25899.96875)
        let result = find_region(25.21875, -20.90625, 25899.96875);
        assert!(result.is_some(), "Sgr A* should be in a known region");
        let r = result.unwrap();
        assert_eq!(r.id, 1, "Sgr A* should be in Galactic Centre (ID 1)");
        assert_eq!(r.name, "Galactic Centre");
    }

    #[test]
    fn colonia_is_identified() {
        // Colonia (Jaques Station) is approximately at (-9530.5, -910.28, 19808.12)
        let result = find_region(-9530.5, -910.28, 19808.12);
        assert!(result.is_some(), "Colonia should be in a known region");
        // Colonia is in the Sagittarius-Carina Arm or nearby region
        let r = result.unwrap();
        assert!(r.id > 0, "Colonia should have a valid region ID");
    }

    #[test]
    fn far_outside_returns_none() {
        // Way outside the galaxy
        let result = find_region(100000.0, 0.0, 100000.0);
        assert!(result.is_none(), "Far outside coordinates should return None");
    }

    #[test]
    fn negative_z_edge() {
        // Test near the bitmap edge
        let result = find_region(0.0, 0.0, -25000.0);
        // Should either return a region or None, but not panic
        let _ = result;
    }

    #[test]
    fn region_name_lookup() {
        assert_eq!(region_name(1), Some("Galactic Centre"));
        assert_eq!(region_name(18), Some("Inner Orion Spur"));
        assert_eq!(region_name(42), Some("The Void"));
        assert_eq!(region_name(0), None);
        assert_eq!(region_name(43), None);
    }

    #[test]
    fn sol_region_groups() {
        // Sol is in Inner Orion Spur (18)
        let groups = region_groups(18);
        assert!(groups.contains(&"orion-cygnus"), "Inner Orion Spur should be in orion-cygnus group");
        assert!(groups.contains(&"sagittarius-carina"), "Inner Orion Spur should be in sagittarius-carina group");
    }

    #[test]
    fn region_in_group_works() {
        // Inner Orion Spur (18) is in orion-cygnus
        assert!(region_in_group(18, "orion-cygnus"));
        // Outer Arm (27) is in outer
        assert!(region_in_group(27, "outer"));
        // Galactic Centre (1) is in center
        assert!(region_in_group(1, "center"));
        // Galactic Centre should NOT be in perseus-core
        assert!(!region_in_group(1, "perseus-core"));
    }
}
