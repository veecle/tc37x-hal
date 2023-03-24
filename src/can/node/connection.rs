//! See https://www.infineon.com/dgdl/Infineon-AURIX_TC3xx_Part2-UserManual-v02_00-EN.pdf?fileId=5546d462712ef9b701717d35f8541d94#%5B%7B%22num%22%3A10218%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C42%2C643%2Cnull%5D
//!
//! Example coding on PIN configuration with some basic sematic control enforced
//! by types
use core::marker::PhantomData;

use tc37x_pac::{PORT_15, PORT_20};
use tc37x_rt::call_without_endinit;

use crate::can::CanModule;

use self::states::{
    Connection, InternalBusConnected, InternalBusDisconnected, PinConnected, PinDisconnected,
};

use super::{CanNode, InConfiguration};

mod states {
    use core::marker::PhantomData;
    use super::CanPin;

    pub struct PinConnected<P: CanPin> {
        marker: PhantomData<P>,
    }

    pub struct PinDisconnected<P: CanPin> {
        marker: PhantomData<P>,
    }

    pub struct InternalBusConnected;

    pub struct InternalBusDisconnected;

    pub struct Connection<P, I> {
        marker: PhantomData<(P, I)>,
    }

    pub type DefaultDisconnected<P> = Connection<PinDisconnected<P>, InternalBusDisconnected>;

    pub trait Connected {}

    impl<P: CanPin> Connected for Connection<PinConnected<P>, InternalBusDisconnected> {}
    impl<P: CanPin> Connected for Connection<PinDisconnected<P>, InternalBusConnected> {}
    impl<P: CanPin> Connected for Connection<PinConnected<P>, InternalBusConnected> {}
}

pub use states::*;

impl<'r, P, R, T, M: CanModule>
    CanNode<'r, Connection<P, InternalBusDisconnected>, InConfiguration, R, T, M>
{
    /// Connect or disconnect this module from the internal loopback bus
    pub fn connect_internal_loopback(
        self,
    ) -> CanNode<'r, Connection<P, InternalBusConnected>, InConfiguration, R, T, M> {
        self.node.npcr.modify(|_, w| w.lbm().bit(true));

        CanNode {
            marker: PhantomData,
            ..self
        }
    }
}

impl<'r, P: CanPin, I, R, T, M: CanModule>
    CanNode<'r, Connection<PinDisconnected<P>, I>, InConfiguration, R, T, M>
{
    #[allow(unused)]
    pub fn set_pins(
        self,
        pin: P,
        port_access: &<P as CanPin>::PeripheralPort,
    ) -> CanNode<'r, Connection<PinConnected<P>, I>, InConfiguration, R, T, M> {
        pin.setup_with(port_access);
        self.node.npcr.modify(|_, w| w.rxsel().variant(pin.rxsel()));

        CanNode {
            marker: PhantomData,
            ..self
        }
    }
}
/// Implementors can be used as a can rx/tx pin for a specific node
pub trait CanPin: Clone {
    type PeripheralPort;
    fn setup_with(&self, port_20: &Self::PeripheralPort);

    fn rxsel(&self) -> u8;
}

#[derive(Default, Clone, Copy)]
pub enum Node0Pin {
    /// Receive Port P20.7, Transmit Port P20.8
    #[default]
    Rxdb = 0b001,
}

impl CanPin for Node0Pin {
    type PeripheralPort = PORT_20;
    fn setup_with(&self, port_20: &PORT_20) {
        match self {
            Node0Pin::Rxdb => {
                call_without_endinit(|| {
                    port_20.iocr4.modify(|_, w| w.pc7().variant(0b10)); // Input pull up
                    port_20.pdr0.modify(|_, w| w.pd7().variant(0b0)); // Strong driver

                    port_20.iocr8.modify(|_, w| w.pc8().variant(0b10000 + 5)); // push-pull output, alternate function 5
                    port_20.pdr1.modify(|_, w| w.pd8().variant(0b1)); // Strong driver, medium edge
                });
            }
        }
    }

    fn rxsel(&self) -> u8 {
        *self as u8
    }
}

#[derive(Default, Clone, Copy)]
pub enum Node1Pin {
    /// Receive Port P15.3, Transmit Port P15.2
    #[default]
    Rxda = 0,
}

impl CanPin for Node1Pin {
    type PeripheralPort = PORT_15;
    fn setup_with(&self, port_15: &PORT_15) {
        match self {
            Node1Pin::Rxda => {
                call_without_endinit(|| {
                    port_15.iocr0.modify(|_, w| w.pc3().variant(0b10)); // Input pull down
                    port_15.pdr0.modify(|_, w| w.pd3().variant(0b0)); // Strong driver

                    port_15.iocr0.modify(|_, w| w.pc2().variant(0b10000 + 5)); // push-pull output, alternate function 5
                    port_15.pdr0.modify(|_, w| w.pd2().variant(0b0)); // Strong driver
                });
            }
        }
    }

    fn rxsel(&self) -> u8 {
        *self as u8
    }
}
