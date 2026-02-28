pub mod domains;
pub mod error;
pub mod types;

pub use error::{CalculationError, InputError, JLawError, RegistryError};
pub use types::{
    FinalAmount, IntermediateAmount, LegalCitation, MultiplyOrder, Rate, RoundingStrategy,
};
