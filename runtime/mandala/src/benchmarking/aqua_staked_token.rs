// This file is part of Acala.

// Copyright (C) 2020-2022 Acala Foundation.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use super::utils::{dollar, set_balance};
use crate::*;

use frame_benchmarking::whitelisted_caller;
use frame_support::traits::OnInitialize;
use frame_system::RawOrigin;

use ecosystem_aqua_staked_token::Vesting;
use orml_traits::MultiLockableCurrency;

const ADAO: CurrencyId = CurrencyId::Token(TokenSymbol::ADAO);
const SADAO: CurrencyId = CurrencyId::Token(TokenSymbol::SADAO);

const VESTING_LOCK_ID: LockIdentifier = *b"aquavest";

runtime_benchmarks! {
	{ Runtime, ecosystem_aqua_staked_token }

	on_initialize {
		let alice = whitelisted_caller();
		set_balance(ADAO, &alice, dollar(ADAO) * 1_000);

		let (block_count, _) = InflationRatePerNBlock::get();
	}: {
		AquaStakedToken::on_initialize(block_count)
	}

	on_initialize_without_inflation {
		let alice = whitelisted_caller();
		set_balance(ADAO, &alice, dollar(ADAO) * 1_000);

		let (block_count, _) = InflationRatePerNBlock::get();
	}: {
		AquaStakedToken::on_initialize(block_count + 1)
	}

	stake {
		let alice = whitelisted_caller();
		set_balance(ADAO, &alice, dollar(ADAO) * 1_000);
		let amount = dollar(ADAO) * 500;
	}: _(RawOrigin::Signed(alice), amount)

	unstake {
		let alice = whitelisted_caller();
		set_balance(SADAO, &alice, dollar(SADAO) * 1_000);
		set_balance(ADAO, &AquaStakedToken::account_id(), dollar(ADAO) * 10_000);
		let amount = dollar(SADAO) * 500;
	}: _(RawOrigin::Signed(alice), amount)

	claim {
		let alice = whitelisted_caller();
		let amount = dollar(SADAO) * 100;
		set_balance(SADAO, &alice, amount);
		Currencies::set_lock(VESTING_LOCK_ID, SADAO, &alice, amount)?;

		let vesting = Vesting {
			unlock_at: 0,
			amount,
		};
		ecosystem_aqua_staked_token::Vestings::<Runtime>::insert(&alice, vesting);

		System::set_block_number(100);
	}: _(RawOrigin::Signed(alice))

	update_unstake_fee_rate {
		let rate = Rate::saturating_from_rational(1, 10);
	}: _(RawOrigin::Root, rate)
}