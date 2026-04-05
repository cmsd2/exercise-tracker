# FIT File Parsing

## Overview

FIT (Flexible and Interoperable Data Transfer) is Garmin's binary file format for recording activity data. The application downloads FIT files during Garmin sync, parses them into structured data, serialises the result as JSON, and stores it in the `fit_data` column of the activities table.

## Parser Design

### Pipeline

```
Garmin API  ->  Binary FIT bytes  ->  fitparser crate  ->  FitDetail struct  ->  JSON string  ->  SQLite
```

1. **Download**: `GarminClient::download_fit_file(activity_id)` fetches the raw binary from `https://connect.garmin.com/download-service/files/activity/{id}`
2. **Parse**: `fitparser::from_bytes(&bytes)` decodes the binary FIT format into `FitDataRecord` objects
3. **Extract**: `parse_fit_data(&bytes)` iterates records, dispatches on message type, and extracts named fields
4. **Serialise**: `serde_json::to_string(&FitDetail)` produces a JSON string
5. **Store**: The JSON is stored as `fit_data TEXT` in the activities table

### Dependencies

- [`fitparser`](https://crates.io/crates/fitparser) v0.7 -- Rust FIT file parser based on Garmin's FIT SDK

### Message Types Extracted

The parser processes four FIT message types:

| MesgNum | Struct | Typical count | Description |
|---|---|---|---|
| `Session` | `FitSession` | 1 | Overall activity summary |
| `Lap` | `FitLap` | 1-N | Per-lap splits |
| `Record` | `FitRecord` | 100s-1000s | Per-second or per-point data |
| `DeviceInfo` | `FitDeviceInfo` | 1-5 | Connected devices |

All other message types (FileId, Event, Activity, etc.) are silently ignored.

## Parsed Fields

### FitSession (Activity Summary)

| Field | Type | Source field(s) |
|---|---|---|
| `sport` | String | `sport` |
| `sub_sport` | String | `sub_sport` |
| `total_elapsed_time` | f64 | `total_elapsed_time` |
| `total_timer_time` | f64 | `total_timer_time` |
| `total_distance` | f64 | `total_distance` |
| `total_calories` | u32 | `total_calories` |
| `avg_heart_rate` | u8 | `avg_heart_rate` |
| `max_heart_rate` | u8 | `max_heart_rate` |
| `avg_cadence` | u8 | `avg_cadence` or `avg_running_cadence` |
| `max_cadence` | u8 | `max_cadence` or `max_running_cadence` |
| `avg_power` | u16 | `avg_power` |
| `max_power` | u16 | `max_power` |
| `total_ascent` | u16 | `total_ascent` |
| `total_descent` | u16 | `total_descent` |
| `avg_speed` | f64 | `avg_speed` or `enhanced_avg_speed` |
| `max_speed` | f64 | `max_speed` or `enhanced_max_speed` |
| `avg_temperature` | i8 | `avg_temperature` |
| `training_stress_score` | f64 | `training_stress_score` |
| `intensity_factor` | f64 | `intensity_factor` |
| `threshold_power` | u16 | `threshold_power` |
| `normalized_power` | u16 | `normalized_power` |
| `swim_stroke` | String | `swim_stroke` |
| `pool_length` | f64 | `pool_length` |
| `num_laps` | u16 | `num_laps` |

### FitLap (Per-Lap Data)

| Field | Type | Source field(s) |
|---|---|---|
| `start_time` | String (RFC 3339) | `start_time` |
| `total_elapsed_time` | f64 | `total_elapsed_time` |
| `total_timer_time` | f64 | `total_timer_time` |
| `total_distance` | f64 | `total_distance` |
| `total_calories` | u32 | `total_calories` |
| `avg_heart_rate` | u8 | `avg_heart_rate` |
| `max_heart_rate` | u8 | `max_heart_rate` |
| `avg_cadence` | u8 | `avg_cadence` or `avg_running_cadence` |
| `max_cadence` | u8 | `max_cadence` or `max_running_cadence` |
| `avg_power` | u16 | `avg_power` |
| `max_power` | u16 | `max_power` |
| `avg_speed` | f64 | `avg_speed` or `enhanced_avg_speed` |
| `max_speed` | f64 | `max_speed` or `enhanced_max_speed` |
| `total_ascent` | u16 | `total_ascent` |
| `total_descent` | u16 | `total_descent` |
| `intensity` | String | `intensity` |
| `lap_trigger` | String | `lap_trigger` |
| `swim_stroke` | String | `swim_stroke` |
| `num_lengths` | u16 | `num_lengths` |
| `avg_stroke_count` | f64 | `avg_stroke_count` |

### FitRecord (Per-Point Data)

| Field | Type | Source field(s) |
|---|---|---|
| `timestamp` | String (RFC 3339) | `timestamp` |
| `position_lat` | f64 | `position_lat` (semicircles) |
| `position_long` | f64 | `position_long` (semicircles) |
| `altitude` | f64 | `altitude` |
| `heart_rate` | u8 | `heart_rate` |
| `cadence` | u8 | `cadence` or `fractional_cadence` |
| `distance` | f64 | `distance` |
| `speed` | f64 | `speed` |
| `power` | u16 | `power` |
| `temperature` | i8 | `temperature` |
| `enhanced_altitude` | f64 | `enhanced_altitude` |
| `enhanced_speed` | f64 | `enhanced_speed` |

Note: GPS coordinates are in FIT's native "semicircles" unit. To convert to degrees: `degrees = semicircles * (180 / 2^31)`.

### FitDeviceInfo

| Field | Type | Source field(s) |
|---|---|---|
| `manufacturer` | String | `manufacturer` |
| `product` | String | `product` or `garmin_product` |
| `serial_number` | u32 | `serial_number` |
| `software_version` | f64 | `software_version` |
| `device_index` | u8 | `device_index` |
| `device_type` | String | `device_type`, `source_type`, or `antplus_device_type` |
| `battery_voltage` | f64 | `battery_voltage` |
| `battery_status` | String | `battery_status` |

## Type Conversion

FIT fields can have various underlying types. The parser uses flexible extraction functions that accept multiple value variants:

```rust
fn get_f64(field) -> Option<f64>   // Float64, Float32, UInt32, SInt32, UInt16, SInt16, UInt8, SInt8
fn get_u8(field) -> Option<u8>     // UInt8, UInt8z, Byte, Enum
fn get_u16(field) -> Option<u16>   // UInt16, UInt16z, UInt8
fn get_u32(field) -> Option<u32>   // UInt32, UInt32z, UInt16, UInt8
fn get_i8(field) -> Option<i8>     // SInt8, UInt8
fn get_string(field) -> Option<String>  // String, or Debug format of other types
fn get_timestamp_string(field) -> Option<String>  // Timestamp -> RFC 3339
```

This handles the variation in how different Garmin devices encode the same logical field.

## FIT Versioning

### Problem

FIT parsing logic may improve over time: new fields extracted, bugs fixed, better handling of edge cases. Activities imported with an older parser version may have incomplete or incorrect `fit_data`.

### Solution

Each activity has a `fit_version` integer column:

- `0` = no FIT data processed (download or parse failed)
- `1` = current parser version (`CURRENT_FIT_VERSION` constant in `commands/garmin.rs`)

When the parser is improved:

1. Increment `CURRENT_FIT_VERSION` in the code
2. Run `garmin_reprocess_fit` command
3. Finds activities where `source = 'garmin' AND fit_version < CURRENT_FIT_VERSION`
4. Re-downloads FIT file from Garmin, re-parses, updates `fit_data` and `fit_version`

This allows iterating on the parser without requiring a full activity re-sync.

### Limitations

- Re-processing requires the Garmin API to still be accessible (token valid, activity still exists)
- Raw FIT files are not cached locally -- re-processing re-downloads each file
- If Garmin deletes an activity, the FIT data cannot be updated

## Storage

### Database Column

`fit_data` is stored as a `TEXT` column containing the JSON serialisation of `FitDetail`. For a typical running activity, this is roughly 50-500 KB depending on duration and recording frequency.

### Performance

- `list_activities()` returns `NULL AS fit_data` to avoid loading large blobs when browsing history
- `get_activity()` loads the full FIT data for a single activity
- FIT data is only written during Garmin sync or re-processing, never during manual activity creation

### Example JSON

```json
{
  "session": {
    "sport": "Running",
    "sub_sport": "Generic",
    "total_elapsed_time": 1823.45,
    "total_timer_time": 1800.12,
    "total_distance": 5012.3,
    "total_calories": 412,
    "avg_heart_rate": 155,
    "max_heart_rate": 178,
    "avg_cadence": 85,
    "avg_speed": 2.78,
    "total_ascent": 42,
    "total_descent": 38,
    ...
  },
  "laps": [
    {
      "start_time": "2026-04-01T07:30:00+01:00",
      "total_elapsed_time": 312.5,
      "total_distance": 1000.0,
      "avg_heart_rate": 148,
      "avg_speed": 3.2,
      ...
    }
  ],
  "records": [
    {
      "timestamp": "2026-04-01T07:30:01+01:00",
      "position_lat": 622879744,
      "position_long": -15728640,
      "heart_rate": 142,
      "cadence": 82,
      "speed": 2.8,
      "distance": 2.8,
      ...
    }
  ],
  "device_info": [
    {
      "manufacturer": "Garmin",
      "product": "Forerunner 265",
      "serial_number": 3987654321,
      "software_version": 15.2
    }
  ]
}
```
