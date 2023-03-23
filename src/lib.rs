//! Initial, unstructured and unstable implementation of an example HAL for AURIX TC37XX
//!
//! Most focus went into the design of
//! - [can]: a basic CAN driver implementation to send/receive frames in non-blocking manner
//! - [clocks]: for PLL & CCU configuration (i.e., to setup the peripheral and system clock)
//!
#![no_std]
#![feature(type_changing_struct_update)]

pub mod can;
pub mod clocks;
pub mod delay;
pub mod frequency;
pub mod time;
