// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for pallet_bounties
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_bounties
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/frame_weight_template.hbs
// --output=./runtime/common/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for pallet_bounties (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bounties::weights::WeightInfo for WeightInfo<T> {
    // Storage: Bounties BountyCount (r:1 w:1)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    // Storage: Bounties Bounties (r:0 w:1)
    fn propose_bounty(d: u32) -> Weight {
        Weight::from_ref_time(36_538_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_000).saturating_mul(d as u64))
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(3 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: Bounties BountyApprovals (r:1 w:1)
    fn approve_bounty() -> Weight {
        Weight::from_ref_time(13_350_000)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn propose_curator() -> Weight {
        Weight::from_ref_time(9_910_000)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unassign_curator() -> Weight {
        Weight::from_ref_time(34_640_000)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn accept_curator() -> Weight {
        Weight::from_ref_time(36_380_000)
            .saturating_add(T::DbWeight::get().reads(2 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn award_bounty() -> Weight {
        Weight::from_ref_time(22_830_000)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn claim_bounty() -> Weight {
        Weight::from_ref_time(103_830_000)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(5 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn close_bounty_proposed() -> Weight {
        Weight::from_ref_time(38_610_000)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(2 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn close_bounty_active() -> Weight {
        Weight::from_ref_time(70_980_000)
            .saturating_add(T::DbWeight::get().reads(4 as u64))
            .saturating_add(T::DbWeight::get().writes(5 as u64))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn extend_bounty_expiry() -> Weight {
        Weight::from_ref_time(24_230_000)
            .saturating_add(T::DbWeight::get().reads(1 as u64))
            .saturating_add(T::DbWeight::get().writes(1 as u64))
    }
    // Storage: Bounties BountyApprovals (r:1 w:1)
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn spend_funds(b: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 252_000
            .saturating_add(Weight::from_ref_time(52_072_000).saturating_mul(b as u64))
            .saturating_add(T::DbWeight::get().reads((3 as u64).saturating_mul(b as u64)))
            .saturating_add(T::DbWeight::get().writes((3 as u64).saturating_mul(b as u64)))
    }
}
