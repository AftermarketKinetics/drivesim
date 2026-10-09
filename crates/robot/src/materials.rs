//! Material name -> density, kg/m³. Values follow the cad-step plugin's
//! table (~/code/cad-step-parser-skills, step_measure.py), plus the
//! combat-robot materials it lacks (AR500, Hardox).

pub fn density(name: &str) -> Option<f64> {
    let key = name.trim().to_ascii_lowercase().replace(' ', "-");
    let g_cm3 = match key.as_str() {
        "aluminium" | "aluminum" | "al6061" => 2.70,
        "al7075" | "7075" => 2.81,
        "steel" | "mild-steel" | "4140" => 7.85,
        "s7" => 7.83,
        "ar500" | "hardox" => 7.85,
        "stainless" | "ss304" | "ss316" => 8.00,
        "titanium" | "ti64" => 4.43,
        "brass" => 8.50,
        "bronze" => 8.80,
        "copper" => 8.96,
        "tungsten" => 19.30,
        "magnesium" => 1.74,
        "abs" => 1.04,
        "pla" => 1.24,
        "petg" => 1.27,
        "nylon" => 1.15,
        "pa12" => 1.01,
        "tpu" => 1.20,
        "delrin" | "pom" => 1.41,
        "pc" | "polycarbonate" => 1.20,
        "uhmw" => 0.94,
        "hdpe" => 0.95,
        "carbon-fiber" => 1.60,
        "g10" | "fr4" => 1.85,
        "rubber" => 1.20,
        "lipo" => 2.10,
        _ => return None,
    };
    Some(g_cm3 * 1000.0)
}

#[cfg(test)]
mod tests {
    use super::density;

    #[test]
    fn names_are_case_and_space_insensitive() {
        assert_eq!(density("AR500"), Some(7850.0));
        assert_eq!(density("Mild Steel"), Some(7850.0));
        assert_eq!(density("unobtainium"), None);
    }
}
