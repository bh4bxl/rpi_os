#[path = "../../arch/aarch64/memory/mmu/translation_table.rs"]
mod arch_translation_table;

pub use arch_translation_table::KernelTranslationTable;
