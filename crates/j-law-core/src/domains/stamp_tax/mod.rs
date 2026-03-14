pub mod calculator;
pub mod context;
pub mod params;
pub mod policy;

pub use calculator::{calculate_stamp_tax, StampTaxBreakdownStep, StampTaxResult};
pub use context::{StampTaxContext, StampTaxDocumentCode, StampTaxFlag};
pub use params::{
    StampTaxAmountUsage, StampTaxBracket, StampTaxChargeMode, StampTaxCitation,
    StampTaxDocumentParams, StampTaxParams, StampTaxSpecialRule,
};
pub use policy::StandardNtaPolicy;
