use core::convert;

use aarch64_cpu::registers::{Readable, Writeable};
use tock_registers::{register_bitfields, registers::InMemoryRegister};

use crate::{
    boards,
    memory::{
        self,
        mmu::{
            arch_mmu::{Granule512MiB, Granule64KiB},
            AccessPermissions, AttributeFields, MemAttributes,
        },
    },
};

// A table descriptor.
register_bitfields! {
    u64,
    STAGE1_TABLE_DESCRIPTOR [
        /// Physical address of the next descriptor.
        NEXT_LEVEL_TABLE_ADDR_64KiB OFFSET(16) NUMBITS(32) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Block = 0,
            Table = 1,
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1,
        ],
    ],
}

// A level 3 page descriptor
register_bitfields! {
    u64,
    STAGE1_PAGE_DESCRIPTOR [
        /// Unprivileged execute-never.
        UXN OFFSET(54) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        /// Privileged execute-never.
        PXN OFFSET(53) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        /// Physical address of the next table descriptor (lvl2) or the page descriptor (lvl3).
        OUTPUT_ADDR_64KiB OFFSET(16) NUMBITS(32) [],

        /// Access flag.
        AF OFFSET(10) NUMBITS(1) [
            False = 0,
            True = 1,
        ],

        /// Shareability field.
        SH OFFSET(8) NUMBITS(2) [
            OuterShareable = 0b10,
            InnerShareable = 0b11,
        ],

        /// Access Permissions.
        AP OFFSET(6) NUMBITS(2) [
            RW_EL1 = 0b00,
            RW_EL1_EL0 = 0b01,
            RO_EL1 = 0b10,
            RO_EL1_EL0 = 0b11,
        ],

        /// Memory attributes index into the MAIR_EL1 register.
        AttrIndx OFFSET(2) NUMBITS(3) [],

        TYPE OFFSET(1) NUMBITS(1) [
            Reserved_Invalid = 0,
            Page = 1,
        ],

        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1,
        ],
    ],
}

/// A table descriptor for 64 KiB aperture.
#[derive(Copy, Clone)]
#[repr(C)]
struct TableDescriptor {
    value: u64,
}

impl TableDescriptor {
    /// Create an instance.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance pointing to the supplied address.
    pub fn from_next_lvl_table_addr(phys_next_lvl_table_addr: usize) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_TABLE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_next_lvl_table_addr >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_TABLE_DESCRIPTOR::NEXT_LEVEL_TABLE_ADDR_64KiB.val(shifted as u64)
                + STAGE1_TABLE_DESCRIPTOR::TYPE::Table
                + STAGE1_TABLE_DESCRIPTOR::VALID::True,
        );

        TableDescriptor { value: val.get() }
    }
}

/// A page descriptor with 64 KiB aperture.
#[derive(Copy, Clone)]
#[repr(C)]
struct PageDescriptor {
    value: u64,
}

impl PageDescriptor {
    /// Create an instance.
    pub const fn new_zeroed() -> Self {
        Self { value: 0 }
    }

    /// Create an instance.
    pub fn from_output_addr(phys_output_addr: usize, attribute_fields: &AttributeFields) -> Self {
        let val = InMemoryRegister::<u64, STAGE1_PAGE_DESCRIPTOR::Register>::new(0);

        let shifted = phys_output_addr as u64 >> Granule64KiB::SHIFT;
        val.write(
            STAGE1_PAGE_DESCRIPTOR::OUTPUT_ADDR_64KiB.val(shifted)
                + STAGE1_PAGE_DESCRIPTOR::AF::True
                + STAGE1_PAGE_DESCRIPTOR::TYPE::Page
                + STAGE1_PAGE_DESCRIPTOR::VALID::True
                + (*attribute_fields).into(),
        );

        Self { value: val.get() }
    }
}

/// Convert the kernel's generic memory attributes to HW-specific attributes of the MMU.
impl convert::From<AttributeFields>
    for tock_registers::fields::FieldValue<u64, STAGE1_PAGE_DESCRIPTOR::Register>
{
    fn from(value: AttributeFields) -> Self {
        // Memory attributes.
        let mut desc = match value.mem_attributes {
            MemAttributes::CacheableDram => {
                STAGE1_PAGE_DESCRIPTOR::SH::InnerShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::NORMAL)
            }
            MemAttributes::Device => {
                STAGE1_PAGE_DESCRIPTOR::SH::OuterShareable
                    + STAGE1_PAGE_DESCRIPTOR::AttrIndx.val(memory::mmu::arch_mmu::mair::DEVICE)
            }
        };

        // Access Permissions.
        desc += match value.acc_perms {
            AccessPermissions::ReadOnly => STAGE1_PAGE_DESCRIPTOR::AP::RO_EL1,
            AccessPermissions::ReadWrite => STAGE1_PAGE_DESCRIPTOR::AP::RW_EL1,
        };

        // The execute-never attribute is mapped to PXN in AArch64.
        desc += match value.execute_never {
            true => STAGE1_PAGE_DESCRIPTOR::PXN::True,
            false => STAGE1_PAGE_DESCRIPTOR::PXN::False,
        };

        desc += STAGE1_PAGE_DESCRIPTOR::UXN::True;

        desc
    }
}

trait StartAddr {
    fn phys_start_addr_u64(&self) -> u64;
    fn phys_start_addr_usize(&self) -> usize;
}

impl<T, const N: usize> StartAddr for [T; N] {
    fn phys_start_addr_u64(&self) -> u64 {
        self as *const T as u64
    }

    fn phys_start_addr_usize(&self) -> usize {
        self as *const _ as usize
    }
}

const NUM_LVL2_TABLES: usize =
    boards::rpi4::memory::mmu::KernelAddrSpace::SIZE >> Granule512MiB::SHIFT;

/// Big monolithic struct for storing the translation tables. Individual levels must be 64 KiB
/// aligned, so the lvl3 is put first.
#[repr(C)]
#[repr(align(65536))]
pub struct FixedSizeTranslationTable<const NUM_TABLES: usize> {
    /// Page descriptors, covering 64 KiB windows per entry.
    lvl3: [[PageDescriptor; 8192]; NUM_TABLES],

    /// Table descriptors, covering 512 MiB windows.
    lvl2: [TableDescriptor; NUM_TABLES],
}

impl<const NUM_TABLES: usize> FixedSizeTranslationTable<NUM_TABLES> {
    /// Create an instance.
    pub const fn new() -> Self {
        // Can't have a zero-sized address space.
        assert!(NUM_TABLES > 0);

        Self {
            lvl3: [[PageDescriptor::new_zeroed(); 8192]; NUM_TABLES],
            lvl2: [TableDescriptor::new_zeroed(); NUM_TABLES],
        }
    }

    /// Iterates over all static translation table entries and fills them at once.
    /// # Safety
    pub unsafe fn populate_tt_entries(&mut self) -> Result<(), &'static str> {
        for (l2_nr, l2_entry) in self.lvl2.iter_mut().enumerate() {
            *l2_entry =
                TableDescriptor::from_next_lvl_table_addr(self.lvl3[l2_nr].phys_start_addr_usize());

            for (l3_nr, l3_entry) in self.lvl3[l2_nr].iter_mut().enumerate() {
                let virt_addr = (l2_nr << Granule512MiB::SHIFT) + (l3_nr << Granule64KiB::SHIFT);

                let (phys_output_addr, attribute_fields) =
                    boards::rpi4::memory::mmu::virt_mem_layout().virt_addr_properties(virt_addr)?;

                *l3_entry = PageDescriptor::from_output_addr(phys_output_addr, &attribute_fields);
            }
        }

        Ok(())
    }

    /// The translation table's base address to be used for programming the MMU.
    pub fn phys_base_address(&self) -> u64 {
        self.lvl2.phys_start_addr_u64()
    }
}

/// A translation table type for the kernel space.
pub type KernelTranslationTable = FixedSizeTranslationTable<NUM_LVL2_TABLES>;
