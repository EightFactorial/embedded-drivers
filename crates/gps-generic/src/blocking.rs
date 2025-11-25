use ef_driver_common::mode::Blocking;
use embedded_io::Read;

use crate::GenericGps;

impl<UART: Read, const N: usize> GenericGps<UART, Blocking, N> {}
