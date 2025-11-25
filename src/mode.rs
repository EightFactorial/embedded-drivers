//! Marker types and traits for driver operation modes.

/// A marker type for asynchronous drivers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Async;
impl DriverMode for Async {}

/// A marker type for blocking drivers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Blocking;
impl DriverMode for Blocking {}

/// A marker trait for driver operation modes.
pub trait DriverMode: sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Blocking {}
    impl Sealed for super::Async {}
}
