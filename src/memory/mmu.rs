use core::{fmt, ops::RangeInclusive};

#[path = "../arch/aarch64/memory/mmu.rs"]
mod arch_mmu;

mod translation_table;

pub use arch_mmu::mmu;

use crate::common;

/// MMU enable errors variants.
#[derive(Debug)]
pub enum MmuEnableError {
    AlreadyEnabled,
    Other(&'static str),
}

/// Memory Management interfaces.
pub mod interface {
    use crate::memory::mmu::MmuEnableError;

    pub trait Mmu {
        /// Called by the kernel during early init. Supposed to take the translation tables from the
        /// `BSP`-supplied `virt_mem_layout()` and install/activate them for the respective MMU.
        unsafe fn enable_mmu_and_caching(&self) -> Result<(), MmuEnableError>;

        /// Returns true if the MMU is enabled, false otherwise.
        fn is_enabled(&self) -> bool;
    }
}

/// Describes the characteristics of a translation granule.
pub struct TranslationGranule<const GRANULE_SIZE: usize>;

/// Describes properties of an address space.
pub struct AddressSpace<const AS_SIZE: usize>;

/// Architecture agnostic translation types.
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Translation {
    Identity,
    Offset(usize),
}

/// Architecture agnostic memory attributes.
#[derive(Copy, Clone)]
pub enum MemAttributes {
    CacheableDram,
    Device,
}

/// Architecture agnostic access permissions.
#[derive(Copy, Clone)]
pub enum AccessPermissions {
    ReadOnly,
    ReadWrite,
}

/// Collection of memory attributes.
#[derive(Copy, Clone)]
pub struct AttributeFields {
    pub mem_attributes: MemAttributes,
    pub acc_perms: AccessPermissions,
    pub execute_never: bool,
}

/// Architecture agnostic descriptor for a memory range.
pub struct TranslationDescriptor {
    pub name: &'static str,
    pub virtual_range: fn() -> RangeInclusive<usize>,
    pub physical_range_translation: Translation,
    pub attribute_fields: AttributeFields,
}

/// Type for expressing the kernel's virtual memory layout.
pub struct KernelVirtualLayout<const NUM_SPECIAL_RANGES: usize> {
    /// The last (inclusive) address of the address space.
    max_virt_addr_inclusive: usize,

    /// Array of descriptors for non-standard (normal cacheable DRAM) memory regions.
    inner: [TranslationDescriptor; NUM_SPECIAL_RANGES],
}

impl fmt::Display for MmuEnableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MmuEnableError::AlreadyEnabled => write!(f, "MMU is already enabled"),
            MmuEnableError::Other(x) => write!(f, "{}", x),
        }
    }
}

impl<const GRANULE_SIZE: usize> TranslationGranule<GRANULE_SIZE> {
    /// The granule's size.
    pub const SIZE: usize = Self::size_checked();

    /// The granule's shift, aka log2(size).
    pub const SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(GRANULE_SIZE.is_power_of_two());

        GRANULE_SIZE
    }
}

impl<const AS_SIZE: usize> AddressSpace<AS_SIZE> {
    /// The address space size.
    pub const SIZE: usize = Self::size_checked();

    /// The address space shift, aka log2(size).
    pub const SIZE_SHIFT: usize = Self::SIZE.trailing_zeros() as usize;

    const fn size_checked() -> usize {
        assert!(AS_SIZE.is_power_of_two());

        // Check for architectural restrictions as well.
        Self::arch_address_space_size_sanity_checks();

        AS_SIZE
    }
}

impl Default for AttributeFields {
    fn default() -> Self {
        Self {
            mem_attributes: MemAttributes::CacheableDram,
            acc_perms: AccessPermissions::ReadWrite,
            execute_never: true,
        }
    }
}

/// Display of a TranslationDescriptor.
impl fmt::Display for TranslationDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = *(self.virtual_range)().start();
        let end = *(self.virtual_range)().end();
        let size = end - start + 1;

        let (size, unit) = common::size_human_readable_ceil(size);

        let attr = match self.attribute_fields.mem_attributes {
            MemAttributes::CacheableDram => "C",
            MemAttributes::Device => "Dev",
        };

        let acc_p = match self.attribute_fields.acc_perms {
            AccessPermissions::ReadOnly => "RO",
            AccessPermissions::ReadWrite => "RW",
        };

        let xn = match self.attribute_fields.execute_never {
            true => "PXN",
            false => "PX",
        };

        write!(
            f,
            "      {:#010x} - {:#010x} | {: >3} {} | {: <3} {} {: <3} | {}",
            start, end, size, unit, attr, acc_p, xn, self.name
        )
    }
}

impl<const NUM_SPECIAL_RANGS: usize> KernelVirtualLayout<{ NUM_SPECIAL_RANGS }> {
    /// Create a new instance.
    pub const fn new(max: usize, layout: [TranslationDescriptor; NUM_SPECIAL_RANGS]) -> Self {
        Self {
            max_virt_addr_inclusive: max,
            inner: layout,
        }
    }

    /// For a virtual address, find and return the physical output address and corresponding
    /// attributes.
    pub fn virt_addr_properties(
        &self,
        virt_addr: usize,
    ) -> Result<(usize, AttributeFields), &'static str> {
        if virt_addr > self.max_virt_addr_inclusive {
            return Err("Address out of range");
        }

        for i in self.inner.iter() {
            if (i.virtual_range)().contains(&virt_addr) {
                let output_addr = match i.physical_range_translation {
                    Translation::Identity => virt_addr,
                    Translation::Offset(a) => a + (virt_addr - ((i.virtual_range)().start())),
                };

                return Ok((output_addr, i.attribute_fields));
            }
        }

        Ok((virt_addr, AttributeFields::default()))
    }

    /// Print the memory layout.
    pub fn print_layout(&self) {
        use crate::info;

        for i in self.inner.iter() {
            info!("{}", i);
        }
    }
}
