use std::io::{Cursor, Read};

use fitparser::profile::field_types::MesgNum;
use fitparser::{FitDataField, Value};
use zip::ZipArchive;

use super::types::{FitDetail, FitDeviceInfo, FitLap, FitRecord, FitSession};

/// Extract raw FIT bytes from a ZIP archive (Garmin returns FIT files inside a ZIP).
/// If the bytes are not a valid ZIP, returns them as-is (may already be raw FIT).
fn extract_fit_from_zip(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let cursor = Cursor::new(bytes);
    let mut archive = match ZipArchive::new(cursor) {
        Ok(a) => a,
        Err(_) => return Ok(bytes.to_vec()), // Not a ZIP — assume raw FIT
    };

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("ZIP entry error: {}", e))?;
        if file
            .name()
            .to_ascii_lowercase()
            .ends_with(".fit")
        {
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf)
                .map_err(|e| format!("ZIP read error: {}", e))?;
            return Ok(buf);
        }
    }

    Err("ZIP archive contains no .fit file".into())
}

pub fn parse_fit_data(bytes: &[u8]) -> Result<FitDetail, String> {
    let fit_bytes = extract_fit_from_zip(bytes)?;
    let records =
        fitparser::from_bytes(&fit_bytes).map_err(|e| format!("FIT parse error: {}", e))?;

    let mut session: Option<FitSession> = None;
    let mut laps: Vec<FitLap> = Vec::new();
    let mut fit_records: Vec<FitRecord> = Vec::new();
    let mut device_info: Vec<FitDeviceInfo> = Vec::new();

    for record in &records {
        match record.kind() {
            MesgNum::Session => {
                session = Some(parse_session(record.fields()));
            }
            MesgNum::Lap => {
                laps.push(parse_lap(record.fields()));
            }
            MesgNum::Record => {
                fit_records.push(parse_record(record.fields()));
            }
            MesgNum::DeviceInfo => {
                device_info.push(parse_device_info(record.fields()));
            }
            _ => {}
        }
    }

    Ok(FitDetail {
        session,
        laps,
        records: fit_records,
        device_info,
    })
}

fn semicircles_to_degrees(sc: f64) -> f64 {
    sc * (180.0 / 2_147_483_648.0)
}

fn get_f64(field: &FitDataField) -> Option<f64> {
    match field.value() {
        Value::Float64(v) => Some(*v),
        Value::Float32(v) => Some(*v as f64),
        Value::UInt32(v) => Some(*v as f64),
        Value::SInt32(v) => Some(*v as f64),
        Value::UInt16(v) => Some(*v as f64),
        Value::SInt16(v) => Some(*v as f64),
        Value::UInt8(v) => Some(*v as f64),
        Value::SInt8(v) => Some(*v as f64),
        _ => None,
    }
}

fn get_u8(field: &FitDataField) -> Option<u8> {
    match field.value() {
        Value::UInt8(v) | Value::UInt8z(v) | Value::Byte(v) | Value::Enum(v) => Some(*v),
        _ => None,
    }
}

fn get_i8(field: &FitDataField) -> Option<i8> {
    match field.value() {
        Value::SInt8(v) => Some(*v),
        Value::UInt8(v) => Some(*v as i8),
        _ => None,
    }
}

fn get_u16(field: &FitDataField) -> Option<u16> {
    match field.value() {
        Value::UInt16(v) | Value::UInt16z(v) => Some(*v),
        Value::UInt8(v) => Some(*v as u16),
        _ => None,
    }
}

fn get_u32(field: &FitDataField) -> Option<u32> {
    match field.value() {
        Value::UInt32(v) | Value::UInt32z(v) => Some(*v),
        Value::UInt16(v) => Some(*v as u32),
        Value::UInt8(v) => Some(*v as u32),
        _ => None,
    }
}

fn get_string(field: &FitDataField) -> Option<String> {
    match field.value() {
        Value::String(s) => Some(s.clone()),
        _ => Some(format!("{:?}", field.value())),
    }
}

fn get_timestamp_string(field: &FitDataField) -> Option<String> {
    match field.value() {
        Value::Timestamp(dt) => Some(dt.to_rfc3339()),
        _ => None,
    }
}

fn parse_session(fields: &[FitDataField]) -> FitSession {
    let mut s = FitSession {
        sport: None,
        sub_sport: None,
        total_elapsed_time: None,
        total_timer_time: None,
        total_distance: None,
        total_calories: None,
        avg_heart_rate: None,
        max_heart_rate: None,
        avg_cadence: None,
        max_cadence: None,
        avg_power: None,
        max_power: None,
        total_ascent: None,
        total_descent: None,
        avg_speed: None,
        max_speed: None,
        avg_temperature: None,
        training_stress_score: None,
        intensity_factor: None,
        threshold_power: None,
        normalized_power: None,
        swim_stroke: None,
        pool_length: None,
        num_laps: None,
    };

    for field in fields {
        match field.name() {
            "sport" => s.sport = get_string(field),
            "sub_sport" => s.sub_sport = get_string(field),
            "total_elapsed_time" => s.total_elapsed_time = get_f64(field),
            "total_timer_time" => s.total_timer_time = get_f64(field),
            "total_distance" => s.total_distance = get_f64(field),
            "total_calories" => s.total_calories = get_u32(field),
            "avg_heart_rate" => s.avg_heart_rate = get_u8(field),
            "max_heart_rate" => s.max_heart_rate = get_u8(field),
            "avg_cadence" | "avg_running_cadence" => s.avg_cadence = get_u8(field),
            "max_cadence" | "max_running_cadence" => s.max_cadence = get_u8(field),
            "avg_power" => s.avg_power = get_u16(field),
            "max_power" => s.max_power = get_u16(field),
            "total_ascent" => s.total_ascent = get_u16(field),
            "total_descent" => s.total_descent = get_u16(field),
            "avg_speed" | "enhanced_avg_speed" => s.avg_speed = get_f64(field),
            "max_speed" | "enhanced_max_speed" => s.max_speed = get_f64(field),
            "avg_temperature" => s.avg_temperature = get_i8(field),
            "training_stress_score" => s.training_stress_score = get_f64(field),
            "intensity_factor" => s.intensity_factor = get_f64(field),
            "threshold_power" => s.threshold_power = get_u16(field),
            "normalized_power" => s.normalized_power = get_u16(field),
            "swim_stroke" => s.swim_stroke = get_string(field),
            "pool_length" => s.pool_length = get_f64(field),
            "num_laps" => s.num_laps = get_u16(field),
            _ => {}
        }
    }

    s
}

fn parse_lap(fields: &[FitDataField]) -> FitLap {
    let mut l = FitLap {
        start_time: None,
        total_elapsed_time: None,
        total_timer_time: None,
        total_distance: None,
        total_calories: None,
        avg_heart_rate: None,
        max_heart_rate: None,
        avg_cadence: None,
        max_cadence: None,
        avg_power: None,
        max_power: None,
        avg_speed: None,
        max_speed: None,
        total_ascent: None,
        total_descent: None,
        intensity: None,
        lap_trigger: None,
        swim_stroke: None,
        num_lengths: None,
        avg_stroke_count: None,
    };

    for field in fields {
        match field.name() {
            "start_time" => l.start_time = get_timestamp_string(field),
            "total_elapsed_time" => l.total_elapsed_time = get_f64(field),
            "total_timer_time" => l.total_timer_time = get_f64(field),
            "total_distance" => l.total_distance = get_f64(field),
            "total_calories" => l.total_calories = get_u32(field),
            "avg_heart_rate" => l.avg_heart_rate = get_u8(field),
            "max_heart_rate" => l.max_heart_rate = get_u8(field),
            "avg_cadence" | "avg_running_cadence" => l.avg_cadence = get_u8(field),
            "max_cadence" | "max_running_cadence" => l.max_cadence = get_u8(field),
            "avg_power" => l.avg_power = get_u16(field),
            "max_power" => l.max_power = get_u16(field),
            "avg_speed" | "enhanced_avg_speed" => l.avg_speed = get_f64(field),
            "max_speed" | "enhanced_max_speed" => l.max_speed = get_f64(field),
            "total_ascent" => l.total_ascent = get_u16(field),
            "total_descent" => l.total_descent = get_u16(field),
            "intensity" => l.intensity = get_string(field),
            "lap_trigger" => l.lap_trigger = get_string(field),
            "swim_stroke" => l.swim_stroke = get_string(field),
            "num_lengths" => l.num_lengths = get_u16(field),
            "avg_stroke_count" => l.avg_stroke_count = get_f64(field),
            _ => {}
        }
    }

    l
}

fn parse_record(fields: &[FitDataField]) -> FitRecord {
    let mut r = FitRecord {
        timestamp: None,
        position_lat: None,
        position_long: None,
        altitude: None,
        heart_rate: None,
        cadence: None,
        distance: None,
        speed: None,
        power: None,
        temperature: None,
        enhanced_altitude: None,
        enhanced_speed: None,
    };

    for field in fields {
        match field.name() {
            "timestamp" => r.timestamp = get_timestamp_string(field),
            "position_lat" => r.position_lat = get_f64(field).map(semicircles_to_degrees),
            "position_long" => r.position_long = get_f64(field).map(semicircles_to_degrees),
            "altitude" => r.altitude = get_f64(field),
            "heart_rate" => r.heart_rate = get_u8(field),
            "cadence" | "fractional_cadence" => {
                if r.cadence.is_none() {
                    r.cadence = get_u8(field);
                }
            }
            "distance" => r.distance = get_f64(field),
            "speed" => r.speed = get_f64(field),
            "power" => r.power = get_u16(field),
            "temperature" => r.temperature = get_i8(field),
            "enhanced_altitude" => r.enhanced_altitude = get_f64(field),
            "enhanced_speed" => r.enhanced_speed = get_f64(field),
            _ => {}
        }
    }

    r
}

fn parse_device_info(fields: &[FitDataField]) -> FitDeviceInfo {
    let mut d = FitDeviceInfo {
        manufacturer: None,
        product: None,
        serial_number: None,
        software_version: None,
        device_index: None,
        device_type: None,
        battery_voltage: None,
        battery_status: None,
    };

    for field in fields {
        match field.name() {
            "manufacturer" => d.manufacturer = get_string(field),
            "product" | "garmin_product" => {
                if d.product.is_none() {
                    d.product = get_string(field);
                }
            }
            "serial_number" => d.serial_number = get_u32(field),
            "software_version" => d.software_version = get_f64(field),
            "device_index" => d.device_index = get_u8(field),
            "device_type" | "source_type" | "antplus_device_type" => {
                if d.device_type.is_none() {
                    d.device_type = get_string(field);
                }
            }
            "battery_voltage" => d.battery_voltage = get_f64(field),
            "battery_status" => d.battery_status = get_string(field),
            _ => {}
        }
    }

    d
}
