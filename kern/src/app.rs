//! Application description and startup.
//!
//! An "application" here is the entire collection of tasks and configuration
//! that customize the generic kernel.

use zerocopy::FromBytes;

use crate::task::Priority;

pub const CURRENT_APP_MAGIC: u32 = 0x1DE_fa7a1;
pub const REGIONS_PER_TASK: usize = 8;

#[derive(Clone, Debug, FromBytes)]
#[repr(C)]
pub struct App {
    /// Reassures the kernel that it is dealing with this kind of an app struct.
    /// Should have the value `CURRENT_APP_MAGIC`.
    pub magic: u32,
    /// Number of tasks. This many `TaskDesc` records will immediately follow
    /// the app header.
    pub task_count: u32,
    /// Number of memory regions in the address space layout. This many
    /// `RegionDesc` records will immediately follow the `TaskDesc` array.
    pub region_count: u32,
    
    /// Reserved expansion space; pads this structure out to 32 bytes. You will
    /// need to adjust this when you add fields above.
    pub zeroed_expansion_space: [u8; 32 - (3 * 4)],
}

#[derive(Clone, Debug, FromBytes)]
#[repr(C)]
pub struct TaskDesc {
    /// Identifies memory regions this task has access to, by index in the
    /// `RegionDesc` table. If the task needs fewer than `REGIONS_PER_TASK`
    /// regions, it should use remaining entries to name a region that confers
    /// no access; by convention, this region is usually entry 0 in the table.
    pub regions: [u8; REGIONS_PER_TASK],
    /// Address of the task's entry point. This is the first instruction that
    /// will be executed whenever the task is (re)started. It must be within one
    /// of the task's memory regions.
    pub entry_point: u32,
    /// Address of the task's initial stack pointer, to be loaded at (re)start.
    /// It must be pointing into or *just past* one of the task's memory
    /// regions.
    pub initial_stack: u32,
    /// Initial priority of this task.
    pub priority: Priority,
    /// Collection of boolean flags controlling task behavior.
    pub flags: TaskFlags,
}

bitflags::bitflags! {
    #[derive(FromBytes)]
    #[repr(transparent)]
    pub struct TaskFlags: u32 {
        const START_AT_BOOT = 1 << 0;
        const RESERVED = !((1 << 0) - 1);
    }
}

/// Description of one memory region. 
#[derive(Clone, Debug, FromBytes)]
#[repr(C)]
pub struct RegionDesc {
    /// Address of start of region. The platform likely has alignment
    /// requirements for this; it must meet them.
    pub base: u32,
    /// Size of region, in bytes. The platform likely has alignment requirements
    /// for this; it must meet them.
    pub size: u32,
    /// Flags describing what can be done with this region.
    pub attributes: RegionAttributes,
    /// Reserved word, must be zero.
    pub reserved_zero: u32,
}

impl RegionDesc {
    /// Tests whether `slice` is fully enclosed by this region.
    pub fn covers<T>(&self, slice: &crate::umem::USlice<T>) -> bool {
        let self_end = self.base.wrapping_add(self.size).wrapping_sub(1) as usize;
        let slice_end = slice.last_byte_addr();

        self_end >= slice.base_addr() && slice_end >= self.base as usize
    }
}
bitflags::bitflags! {
    #[derive(FromBytes)]
    #[repr(transparent)]
    pub struct RegionAttributes: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const DEVICE = 1 << 3;
        const RESERVED = !((1 << 4) - 1);
    }
}
