// Sapphillon
// SPDX-FileCopyrightText: 2025 Yuta Takahashi
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

//! The `entity` crate provides the core data structures used throughout Sapphillon.
//!
//! This crate defines the database models and the logic for converting them to and from
//! the gRPC types used by the `sapphillon_core` library.

pub mod convert;
pub mod entity;

pub use entity::provider;

#[allow(unused)]
use convert::*;
