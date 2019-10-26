///! This module defines a packed representation for floating point numbers where
///! the layout consists of a header word followed by a raw f64 value.
///!
///! Where supported, the immediate representation should be preferred.
#[cfg(target_arch = "x86_64")]
compile_error!("Packed floats should not be compiled on x86_64, this architecture uses immediate floats!");

use core::fmt;
use core::cmp;
use core::convert::TryFrom;

use crate::erts::HeapAlloc;
use crate::borrow::CloneToProcess;
use crate::erts::exception::system::Alloc;

use crate::erts::term::prelude::{Term, TypedTerm, Header, StaticHeader, TypeError};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Float {
    header: Header<Float>,
    value: f64,
}
impl Float {
    #[inline]
    pub fn new(value: f64) -> Self {
        Self {
            header: Default::default(),
            value: Self::clamp_value(value),
        }
    }

    #[inline(always)]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

impl StaticHeader for Float { const TAG: Term = Term::HEADER_FLOAT; }

impl Eq for Float {}
impl PartialEq for Float {
    #[inline]
    fn eq(&self, other: &Float) -> bool {
        self.value == other.value
    }
}
impl PartialOrd for Float {
    #[inline]
    fn partial_cmp(&self, other: &Float) -> Option<cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl fmt::Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Float")
            .field("header", &self.header)
            .field("value", &self.value)
            .finish()
    }
}

impl CloneToProcess for Float {
    #[inline]
    fn clone_to_heap<A>(&self, heap: &mut A) -> Result<Term, Alloc>
    where
        A: ?Sized + HeapAlloc,
    {
        unsafe {
            let layout = Layout::for_value(self);
            let ptr = heap.alloc_layout(layout)?.as_ptr() as *mut Self;
            ptr::copy_nonoverlapping(self as *const Self, ptr, 1);
            Ok(ptr.into())
        }
    }
}

impl TryFrom<TypedTerm> for Boxed<Float> {
    type Error = TypeError;

    fn try_from(typed_term: TypedTerm) -> Result<Self, Self::Error> {
        match typed_term {
            TypedTerm::Float(float) => Ok(float),
            _ => Err(TypeError),
        }
    }
}

impl TryFrom<TypedTerm> for Float {
    type Error = TypeError;

    fn try_from(typed_term: TypedTerm) -> Result<Self, Self::Error> {
        match typed_term {
            TypedTerm::Float(float) => Ok(*float.as_ptr()),
            _ => Err(TypeError),
        }
    }
}