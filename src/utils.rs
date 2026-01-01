use napi::bindgen_prelude::*;
use napi::{Result, Error, Status, ValueType};

pub fn to_i64(value: Unknown) -> Result<i64> {
    match value.get_type()? {
        ValueType::Number => value.coerce_to_number()?.get_int64(),
        ValueType::BigInt => {
            // If BigInt is available, we try to use it.
            // But if the compiler fails on ValueType::BigInt, we might need another way.
            // For now, let's try the string conversion as a robust fallback/utility.
            let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
            s.parse::<i64>().map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Failed to parse BigInt/Number from string: {}", e),
                )
            })
        }
        _ => {
            // Final fallback: try string anyway
            let s = value.coerce_to_string()?.into_utf8()?.into_owned()?;
            s.parse::<i64>().map_err(|_| {
                Error::new(
                    Status::InvalidArg,
                    "Handle must be a number or a bigint-compatible string".to_string(),
                )
            })
        }
    }
}
