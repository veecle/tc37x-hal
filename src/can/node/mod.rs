use core::marker::PhantomData;

use tc37x_pac::{can0, CAN0};
use tc37x_rt::block_while_nops;

use self::connection::{Connected, DefaultDisconnected};

use super::{timing::CanBitrate, CanModule};

pub mod connection;
pub mod error;
pub mod receive;
pub mod transceive;

/// Generalized node over supported implementations based on [`NodeInstance`], with basic
/// type-state tracking via type argument S.
///
/// For now this is super unsafe and hardcoded (no checks, only CAN0, may hangs, etc.), will
/// be improved in next iteration (with better PAC). Also, it makes sense to continue here
/// when we have async, so we can use our own HAL and only support that
pub struct CanNode<'r, Connection, S, TxConfig, RxConfig, M: CanModule> {
    /// The node of the can module
    node: &'r can0::NODE,
    /// Configuration related to transceiving
    tx_dedicated_config: TxConfig,
    /// Configuration related to receiving
    rx_fifo0_config: RxConfig,
    /// Marker for type states
    marker: PhantomData<(Connection, S, M)>,
}

impl<'r, M: CanModule>
    CanNode<
        'r,
        DefaultDisconnected<connection::Node0Pin>,
        InConfiguration,
        transceive::NoTx,
        receive::NoRx,
        M,
    >
{
    pub(super) fn node0(can0: &'r CAN0) -> Self {
        if can0.node0().cccr.read().init().bit_is_set() {
            defmt::warn!("Node 0 appears to be in configuration mode already, resetting node");
            can0.node0().disable_init();
        }

        can0.node0().enable_init();

        Self {
            node: can0.node0(),
            tx_dedicated_config: transceive::NoTx,
            rx_fifo0_config: receive::NoRx,
            marker: PhantomData,
        }
    }
}

impl<'r, M: CanModule>
    CanNode<
        'r,
        DefaultDisconnected<connection::Node1Pin>,
        InConfiguration,
        transceive::NoTx,
        receive::NoRx,
        M,
    >
{
    pub(super) fn node1(can0: &'r CAN0) -> Self {
        if can0.node1.cccr.read().init().bit_is_set() {
            defmt::warn!("Node 1 appears to be in configuration mode already, resetting node");
            can0.node1.disable_init();
        }

        can0.node1.enable_init();

        Self {
            node: &can0.node1,
            tx_dedicated_config: transceive::NoTx,
            rx_fifo0_config: receive::NoRx,
            marker: PhantomData,
        }
    }
}

impl<'r, C: Connected, AnyRx, AnyTx, M: CanModule>
    CanNode<'r, C, InConfiguration, AnyRx, AnyTx, M>
{
    /// This will enable the node by clearing INIT & CCE
    ///
    /// We only allow this to be done when _something_ is connected to the node,
    /// otherwise the node will go into error mode immediatley.
    pub fn finalize(self) -> CanNode<'r, C, Running, AnyRx, AnyTx, M> {
        self.node.disable_init();

        CanNode {
            marker: PhantomData,
            ..self
        }
    }
}

impl<'r, AnyConnection, AnyRx, AnyTx, M: CanModule>
    CanNode<'r, AnyConnection, InConfiguration, AnyRx, AnyTx, M>
{
    /// Set already correctly computed bitrate
    pub fn set_bitrate(self, cfg: &CanBitrate) -> Self {
        defmt::trace!("Using bitrate configuration {}", cfg);
        self.node.nbtp.modify(|_, w| {
            w.nsjw()
                .variant(cfg.sync_jump_width() - 1)
                .ntseg1()
                .variant(cfg.tseg1() - 1)
                .ntseg2()
                .variant(cfg.tseg2() - 1)
                .nbrp()
                .variant(cfg.pre_scaler() - 1)
        });
        self
    }
}

/// In configuration state
pub struct InConfiguration;

/// Running state
pub struct Running;

pub trait NodeExt {
    /// Helper method to put the node into configuration mode (set INIT & CCE flags)
    fn enable_init(&self);

    /// Helper method to enable the node operation (clear INIT & CCE flags)
    fn disable_init(&self);
}

impl NodeExt for can0::NODE {
    fn enable_init(&self) {
        // Clear Init
        while self.cccr.read().init().bit_is_clear() {
            self.cccr.modify(|_, w| w.init().set_bit());
        }
        // Clear CCE
        while self.cccr.read().cce().bit_is_clear() {
            self.cccr.modify(|_, w| w.cce().set_bit());
        }
    }

    fn disable_init(&self) {
        // Clear CCE
        block_while_nops!(
            {
                self.cccr.modify(|_, w| w.cce().clear_bit());
                self.cccr.read().cce().bit_is_clear()
            },
            "Cannot clear cce flag"
        );
        block_while_nops!(
            {
                self.cccr.modify(|_, w| w.init().clear_bit());
                self.cccr.read().init().bit_is_clear()
            },
            "Cannot clear init flag"
        );
    }
}
