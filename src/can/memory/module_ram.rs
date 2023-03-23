//! Simple example on how to use owned memory and types in Rust.
//! This can be improved in many ways... is just here to given an idea
use core::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit};
use defmt::Format;
use tc37x_pac::can0::node::{
    rxesc::{self},
    txesc,
};

use crate::can::CanModuleRAM;

/// Helper to work with Node memory areas in a somewhat checked and owned way: this is an example
/// on how we can exploit Rust's owned memory and type-system to avoid working with 'raw' pointers
/// and/or unchecked memory
///
/// This follows Rust's informal Builder pattern
pub struct NodeMemoryBuilder<'a, M: CanModuleRAM> {
    /// The offset from where on memory is still free. The internal implementation should not exceed `M::RAM_SIZE`
    free_offset: usize,
    /// Marker for lifetime & object
    marker: PhantomData<&'a M>,
}

impl<'a, M: CanModuleRAM> Format for NodeMemoryBuilder<'a, M> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "NodeMemoryBuilder {{ address: 0x{:X}, used: {}, unused: {} }}",
            M::RAM_LOCATION,
            self.free_offset,
            M::RAM_SIZE - self.free_offset
        )
    }
}

impl<'a, M: CanModuleRAM> NodeMemoryBuilder<'a, M> {
    /// Wrap the module memory in this structure
    ///
    /// # Safety
    /// Callers must make sure that this method is only called once during the lifetime of 'a.
    pub unsafe fn steal_module_mem() -> NodeMemoryBuilder<'a, M> {
        NodeMemoryBuilder {
            free_offset: 0,
            marker: PhantomData,
        }
    }

    /// Create a sequence of elements from the stilll available memory. This effectively
    /// reserves (at least) `size_of(T) * num` bytes. The resulting array will be
    /// aligned according to T
    ///
    /// TODO: What happens if size_of(T) is not a multiple of its alignment size?
    ///
    /// If the available memory is too little, this returns None
    pub fn take<T: Sized + Default>(&mut self, num: usize) -> Option<NodeMemory<'a, T, M>> {
        assert!(num <= 32, "Buffers can only be of size 32");

        let start_address = (M::RAM_LOCATION as usize).checked_add(self.free_offset)?;

        // We need to make sure that the buffer we acquired is aligned for T
        let misaligned_by = start_address % core::mem::align_of::<T>();

        let padding = if misaligned_by > 0 {
            core::mem::align_of::<T>() - misaligned_by
        } else {
            0
        };

        let extra_bytes_required = core::mem::size_of::<T>().checked_mul(num)?;
        if self.free_offset + padding > isize::MAX as usize {
            // Offset too big for core::ptr::add
            return None;
        }
        // SAFETY: conditions checked above
        let buffer_address = unsafe { M::RAM_LOCATION.add(self.free_offset + padding) };

        if extra_bytes_required > isize::MAX as usize {
            // Array too big for core::slice::from_raw_parts_mut
            return None;
        }

        if self.free_offset + padding + extra_bytes_required > M::RAM_SIZE {
            // End of array exceeds module ram
            return None;
        }

        // All checks are done, the following operations will succeed and thus
        // we can modify the internal offset
        let in_module_offset = self.free_offset + padding;
        self.free_offset += padding + extra_bytes_required;

        // # Safety
        // According to `from_raw_parts_mut` this safe since
        // * we checked that `buffer_address` is aligned properly, also there is enough memory for `num` objects
        // * since the array contains MaybeUninit<T>, any content is fine
        // * we moved self.free_offset which guards the memory we are using here
        // * we checked that `extra_bytes_required` does not exceed `isize::MAX`
        let buffer = unsafe {
            core::slice::from_raw_parts_mut(buffer_address as *mut MaybeUninit<UnsafeCell<T>>, num)
        };

        for init in buffer.iter_mut() {
            // BUG?: using MaybeUninit::write does not work, apparently because
            // it uses 64 bit writes under the hood which does not work for CAN
            // message ram
            let pointer_to_inner = init.as_mut_ptr();
            // # Safety
            // Pointer is valid and nothing is to be dropped
            unsafe { core::ptr::write_volatile(pointer_to_inner, Default::default()) };
        }
        // # Safety
        // We initialized the elements and know that sizes/pointers are valid.
        // MaybeUninit is a marker type, hence it is zero-sized
        let buffer = unsafe {
            core::mem::transmute::<&mut [MaybeUninit<UnsafeCell<T>>], &[UnsafeCell<T>]>(buffer)
        };

        Some(NodeMemory {
            buffer,
            in_module_offset,
            marker: PhantomData,
        })
    }

    /// Variant of take that unwraps the Some(_) in a checked manner
    pub fn take_expect<T: Sized + Default>(&mut self, num: usize) -> NodeMemory<'a, T, M> {
        self.take(num).expect("No memory")
    }
}

/// Wrapper around a slice of "unsafe" memory that is shared between the driver & user
///
/// Accesses are protected and validated
pub struct NodeMemory<'a, E: Sized, M: CanModuleRAM> {
    /// This slice's length never exceeds 32
    buffer: &'a [UnsafeCell<E>],
    in_module_offset: usize,
    marker: PhantomData<M>,
}

impl<'a, E: Sized, M: CanModuleRAM> NodeMemory<'a, E, M> {
    /// Return a &mut slice to an element given the index, returning None if out-of-range
    ///
    /// # Safety
    /// The item at the given index must not be referenced anyhwere else.
    pub unsafe fn get(&self, idx: u8) -> Option<&mut E> {
        if idx as usize >= self.buffer.len() {
            None
        } else {
            Some(&mut *self.buffer[idx as usize].get())
        }
    }
    /// Return the relative address of this block
    pub fn in_module_offset(&self) -> usize {
        self.in_module_offset
    }

    /// The number of elements in this slice
    ///
    /// Guaranteed to be less or equal to 32
    pub fn elements(&self) -> u8 {
        self.buffer.len() as u8
    }
}

impl<'a, E: Sized, M: CanModuleRAM> Format for NodeMemory<'a, E, M> {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "NodeMemory {{ address: 0x{:X}, element_count: {}}}",
            self.buffer.as_ptr(),
            self.buffer.len()
        )
    }
}

mod sealed {
    pub trait Buffer {}
}

pub trait CanBuffer: sealed::Buffer + Default + AsMut<[u8]> + AsRef<[u8]> {
    const BUFFER_SIZE: usize;

    type BufferSize: Into<rxesc::F0DS_A> + Into<rxesc::F1DS_A> + Into<txesc::TBDS_A>;

    fn buffer_size() -> Self::BufferSize;
}

macro_rules! create_buffer_types {
    ($(($name:ident, $size:expr, $f0_size:ident, $f1_size:ident, $tx_dedicated_size:ident)),*) => {
        pub enum Sizes {
            $($name),*
        }

        impl From<Sizes> for rxesc::F0DS_A {
            fn from(value: Sizes) -> Self {
                match value {
                    $(Sizes::$name => rxesc::F0DS_A::$f0_size),*
                }
            }
        }
        impl From<Sizes> for rxesc::F1DS_A {
            fn from(value: Sizes) -> Self {
                match value {
                    $(Sizes::$name => rxesc::F1DS_A::$f1_size),*
                }
            }
        }
        impl From<Sizes> for txesc::TBDS_A {
            fn from(value: Sizes) -> Self {
                match value {
                    $(Sizes::$name => txesc::TBDS_A::$tx_dedicated_size),*
                }
            }
        }
        $(
            #[derive(Default)]
            #[repr(C)]
            pub struct $name {
                pub(super) data: [u8; $size]
            }

            impl sealed::Buffer for $name {}

            impl AsRef<[u8]> for $name {
                fn as_ref(&self) -> &[u8] {
                    &self.data
                }
            }

            impl AsMut<[u8]> for $name {
                fn as_mut(&mut self) -> &mut [u8] {
                    &mut self.data
                }
            }

            impl CanBuffer for $name {
                const BUFFER_SIZE: usize = $size;

                type BufferSize = Sizes;

                fn buffer_size() -> Sizes {
                    Sizes::$name
                }
            }
        )*
    };
}

create_buffer_types!((BufferSize8, 8, BUFFER_SIZE8, BUFFER_SIZE8, BUFFER_SIZE8));
