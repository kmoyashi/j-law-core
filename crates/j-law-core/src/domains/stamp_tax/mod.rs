pub mod calculator;
pub mod context;
pub mod params;
pub mod policy;

pub use calculator::{calculate_stamp_tax, StampTaxResult};
pub use context::{StampTaxContext, StampTaxDocumentKind, StampTaxFlag};
pub use params::{StampTaxBracket, StampTaxDocumentParams, StampTaxParams};
pub use policy::StandardNtaPolicy;
