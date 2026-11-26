//! Avaliação de equipamentos (PoE1), em duas partes independentes:
//!
//! - **Qualidade do item** ([`quality`]): quão bons são os rolls/tiers e o potencial de
//!   crafting — vale para o mercado, sem depender da build.
//! - **Upgrade** ([`upgrade`]): o anúncio é MELHOR, PIOR ou EQUIVALENTE ao seu item para a
//!   build, comparando o conjunto de stats.
//!
//! Os dados (únicos, faixas, tiers, tags) vêm da porta [`knowledge::GameKnowledge`]; os pesos
//! e limiares, de [`knowledge::Rules`]. Mod sem dados suficientes é **inconclusivo**: nunca
//! é tratado como roll perfeito.

pub mod knowledge;
pub mod mods;
pub mod profile;
pub mod quality;
pub mod stat;
pub mod unique;
pub mod upgrade;

#[cfg(test)]
pub(crate) mod test_support;

pub use knowledge::{GameKnowledge, Rules};
pub use mods::{InconclusiveReason, ModEvaluation, ModStatus};
pub use profile::{BuildChoice, BuildDetails, BuildInput, BuildProfile, BuildSummary};
pub use quality::{evaluate_item, Confidence, Grade, ItemQuality};
pub use unique::UniqueMatch;
pub use upgrade::{compare_items, UpgradeVerdict, Verdict};
