/* main.rs */
pub const DO_LOAD: bool = true;
pub const DO_CLUSTER: bool = true;
pub const FORCE_SORT: bool = false;

pub const RAW_PATH: &str = "raw_strips";
pub const CUT_PATH: &str = "cut_strips";
pub const PNG_PATH: &str = "result.png";

/* cutter.rs */
pub const CUTTER_THREADS: usize = 6;

/* sorter.rs */
pub const SORT_THREADS: usize = 6;
pub const MAX_GRADIENT: f32 = 1.0;
pub const CONFIDENCE_BONUS: f32 = 1.0;
pub const MIN_CONFIDENCE: f32 = 0.0;
pub const SQRT_COUNT: u32 = 6;