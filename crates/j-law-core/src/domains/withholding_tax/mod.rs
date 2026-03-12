pub mod calculator;
pub mod context;
pub mod params;
pub mod policy;

pub use calculator::{calculate_withholding_tax, WithholdingTaxResult, WithholdingTaxStep};
pub use context::{WithholdingTaxCategory, WithholdingTaxContext, WithholdingTaxFlag};
pub use params::{WithholdingTaxCategoryParams, WithholdingTaxMethod, WithholdingTaxParams};
pub use policy::StandardWithholdingTaxPolicy;
