pub mod calculator;
pub mod context;
pub mod params;
pub mod policy;

pub use calculator::{calculate_stamp_tax, StampTaxResult};
pub use context::{StampTaxContext, StampTaxFlag};
pub use params::{StampTaxBracket, StampTaxParams};
pub use policy::StandardNtaPolicy;
