//! Autogenerated weights for module_cdp_engine
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-03-01, STEPS: [50, ], REPEAT: 20, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB
//! CACHE: 128

// Executed Command:
// target/release/acala
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=*
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./runtime/mandala/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for module_cdp_engine.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> module_cdp_engine::WeightInfo for WeightInfo<T> {
	fn set_collateral_params() -> Weight {
		(68_732_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn set_global_params() -> Weight {
		(21_068_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn liquidate_by_auction() -> Weight {
		(392_660_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(28 as Weight))
			.saturating_add(T::DbWeight::get().writes(17 as Weight))
	}
	fn liquidate_by_dex() -> Weight {
		(479_844_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(29 as Weight))
			.saturating_add(T::DbWeight::get().writes(15 as Weight))
	}
	fn settle() -> Weight {
		(183_906_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(11 as Weight))
			.saturating_add(T::DbWeight::get().writes(8 as Weight))
	}
}