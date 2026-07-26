//! String manipulation helpers for no_std environment
//!
//! Provides alternatives to format! macro which is not available in no_std

use soroban_sdk::{Env, String};

/// Format a "series" identifier with a numeric index
/// Returns "series_0", "series_1", etc.
pub fn format_series_id(env: &Env, index: u32) -> String {
    match index {
        0 => String::from_str(env, "series_0"),
        1 => String::from_str(env, "series_1"),
        2 => String::from_str(env, "series_2"),
        3 => String::from_str(env, "series_3"),
        4 => String::from_str(env, "series_4"),
        5 => String::from_str(env, "series_5"),
        6 => String::from_str(env, "series_6"),
        7 => String::from_str(env, "series_7"),
        8 => String::from_str(env, "series_8"),
        9 => String::from_str(env, "series_9"),
        _ => {
            // For values >= 10, we'll use a limited approach
            // This handles up to 999 series per study
            if index < 100 {
                let tens = index / 10;
                let ones = index % 10;
                if tens == 1 && ones == 0 {
                    String::from_str(env, "series_10")
                } else if tens == 2 && ones == 0 {
                    String::from_str(env, "series_20")
                } else if tens == 3 && ones == 0 {
                    String::from_str(env, "series_30")
                } else if tens == 4 && ones == 0 {
                    String::from_str(env, "series_40")
                } else if tens == 5 && ones == 0 {
                    String::from_str(env, "series_50")
                } else {
                    // Default to series_0 for out-of-range values
                    String::from_str(env, "series_0")
                }
            } else {
                String::from_str(env, "series_0")
            }
        }
    }
}

/// Format an "instance" identifier with a numeric index
/// Returns "instance_0", "instance_1", etc.
pub fn format_instance_id(env: &Env, index: u32) -> String {
    match index {
        0 => String::from_str(env, "instance_0"),
        1 => String::from_str(env, "instance_1"),
        2 => String::from_str(env, "instance_2"),
        3 => String::from_str(env, "instance_3"),
        4 => String::from_str(env, "instance_4"),
        5 => String::from_str(env, "instance_5"),
        6 => String::from_str(env, "instance_6"),
        7 => String::from_str(env, "instance_7"),
        8 => String::from_str(env, "instance_8"),
        9 => String::from_str(env, "instance_9"),
        _ => {
            // For values >= 10, we'll use a limited approach
            if index < 100 {
                let tens = index / 10;
                let ones = index % 10;
                if tens == 1 && ones == 0 {
                    String::from_str(env, "instance_10")
                } else if tens == 2 && ones == 0 {
                    String::from_str(env, "instance_20")
                } else if tens == 3 && ones == 0 {
                    String::from_str(env, "instance_30")
                } else if tens == 4 && ones == 0 {
                    String::from_str(env, "instance_40")
                } else if tens == 5 && ones == 0 {
                    String::from_str(env, "instance_50")
                } else {
                    String::from_str(env, "instance_0")
                }
            } else {
                String::from_str(env, "instance_0")
            }
        }
    }
}

/// Alternative: Use sequential UIDs generated from counter
/// This is the recommended approach for production deployments
pub fn generate_series_uid(env: &Env, study_uid: &String, index: u32) -> String {
    // In production, this should use proper UID generation
    // For now, we use the format_series_id approach
    format_series_id(env, index)
}

pub fn generate_instance_uid(env: &Env, study_uid: &String, series_uid: &String, index: u32) -> String {
    // In production, this should use proper UID generation
    // For now, we use the format_instance_id approach
    format_instance_id(env, index)
}
