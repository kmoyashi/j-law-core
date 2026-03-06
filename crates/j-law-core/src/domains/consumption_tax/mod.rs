pub mod calculator;
pub mod context;
pub mod params;
pub mod policy;

pub use calculator::{calculate_consumption_tax, ConsumptionTaxResult};
pub use context::{ConsumptionTaxContext, ConsumptionTaxFlag};
pub use params::{ConsumptionTaxParams, ConsumptionTaxRate};
pub use policy::StandardConsumptionTaxPolicy;
