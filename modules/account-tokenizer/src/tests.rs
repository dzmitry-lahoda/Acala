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

//! Unit tests for example module.

#![cfg(test)]

use crate::mock::*;
use crate::Weight;
use frame_support::{
	assert_noop, assert_ok,
	dispatch::DispatchError,
	traits::{
		tokens::nonfungibles::{Inspect, Mutate},
		Hooks,
	},
};
use hex_literal::hex;
use module_support::CreateExtended;

use sp_runtime::{
	traits::{AccountIdConversion, BadOrigin},
	ModuleError,
};

const CALL_WEIGHT: Weight = u64::MAX;

#[test]
fn can_create_nft() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			// create a NFT so the class ID isn't 0
			assert_ok!(ModuleNFT::create_class(
				Origin::signed(ALICE),
				Default::default(),
				Default::default(),
				Default::default(),
			));

			assert_eq!(ModuleNFT::next_class_id(), 1);
			let event = Event::ModuleNFT(module_nft::Event::CreatedClass {
				owner: NftPalletId::get().into_sub_account(1),
				admin: AccountTokenizerPalletId::get().into_account(),
				class_id: 1,
			});

			// on runtime upgrade can create new NFT class
			AccountTokenizer::on_runtime_upgrade();
			assert_eq!(AccountTokenizer::nft_class_id(), 1);
			System::assert_last_event(event.clone());

			// Will not re-create the runtime NFT class.
			AccountTokenizer::on_runtime_upgrade();

			assert_eq!(AccountTokenizer::nft_class_id(), 1);
			System::assert_last_event(event);
		});
}

#[test]
fn can_send_mint_request() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			// on runtime upgrade can create new NFT class
			AccountTokenizer::on_runtime_upgrade();

			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);
			System::assert_last_event(Event::Proxy(pallet_proxy::Event::AnonymousCreated {
				anonymous: proxy.clone(),
				who: ALICE,
				proxy_type: Default::default(),
				disambiguation_index: 0,
			}));

			assert_eq!(ForeignStateOracle::next_query_id(), 0);
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));

			System::assert_last_event(Event::AccountTokenizer(crate::Event::MintRequested {
				account: proxy,
				who: ALICE,
			}));
			// free_balance = 1000 - 1(mint fee) - 1(query fee) - 10 (deposit)
			assert_eq!(Balances::free_balance(&ALICE), dollar(988));
			assert_eq!(Balances::reserved_balance(&ALICE), dollar(10));
			assert_eq!(Balances::free_balance(&TREASURY), dollar(1));

			assert_eq!(ForeignStateOracle::next_query_id(), 1);

			assert!(ForeignStateOracle::query_requests(0).is_some());
		});
}

#[test]
fn can_mint_account_token_nft() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Send a mint request.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert!(ForeignStateOracle::query_requests(0).is_some());

			// Dispatch the request to accept the mint.
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));

			assert_eq!(ModuleNFT::owner(&0, &0), Some(ALICE));
			assert_eq!(AccountTokenizer::minted_account(proxy.clone()), Some(0));
			let events = System::events();
			assert_eq!(
				events[events.len() - 2].event,
				Event::AccountTokenizer(crate::Event::AccountTokenMinted {
					owner: ALICE,
					account: proxy,
					token_id: 0,
				})
			);

			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 0,
					task_result: Ok(()),
				},
			));

			// Deposit is returned to the owner after mint is successful
			assert_eq!(Balances::free_balance(&ALICE), dollar(998));
			assert_eq!(Balances::reserved_balance(&ALICE), 0);
			assert_eq!(Balances::free_balance(&TREASURY), dollar(1));
		});
}

#[test]
fn can_handle_bad_oracle_data() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();

			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Send a mint request.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy,
				ALICE.clone(),
				1,
				0,
				0
			));
			assert!(ForeignStateOracle::query_requests(0).is_some());

			// Dispatch the request to accept the burn.
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![],
				CALL_WEIGHT,
			));

			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 0,
					task_result: Err(DispatchError::Module(ModuleError {
						index: 6u8,
						error: 2u8,
						message: Some("BadOracleData"),
					})),
				},
			));
		});
}

#[test]
fn can_reject_mint_request() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Send a mint request.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert!(ForeignStateOracle::query_requests(0).is_some());

			// Dispatch the request to reject the mint.
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![0],
				CALL_WEIGHT,
			));

			assert_eq!(ModuleNFT::owner(&0, &0), None);
			assert_eq!(AccountTokenizer::minted_account(proxy), None);

			let events = System::events();
			assert_eq!(
				events[events.len() - 2].event,
				Event::AccountTokenizer(crate::Event::MintRequestRejected { requester: ALICE })
			);

			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 0,
					task_result: Ok(()),
				},
			));

			// Deposit is repatriated to the treasury due to the rejection of the request.
			assert_eq!(Balances::free_balance(&ALICE), dollar(988));
			assert_eq!(Balances::reserved_balance(&ALICE), 0);
			assert_eq!(Balances::free_balance(&TREASURY), dollar(11));
		});
}

#[test]
fn confirm_request_cannot_be_called_via_extrinsic() {
	ExtBuilder::default().build().execute_with(|| {
		AccountTokenizer::on_runtime_upgrade();

		assert_noop!(
			AccountTokenizer::confirm_mint_request(Origin::signed(ALICE), ALICE, PROXY),
			BadOrigin
		);
		assert_noop!(
			AccountTokenizer::confirm_mint_request(Origin::root(), ALICE, PROXY),
			BadOrigin
		);
		assert_noop!(
			AccountTokenizer::confirm_mint_request(Origin::signed(ORACLE), ALICE, PROXY),
			BadOrigin
		);
	});
}

#[test]
fn can_burn_account_token_nft() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Mint the NFT.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));

			assert_eq!(ModuleNFT::owner(&0, &0), Some(ALICE));
			assert_eq!(AccountTokenizer::minted_account(proxy.clone()), Some(0));

			// Burn the NFT
			// only the owner of the NFT can burn the token
			assert_noop!(
				AccountTokenizer::request_redeem(Origin::signed(proxy.clone()), proxy.clone(), ALICE),
				crate::Error::<Runtime>::CallerUnauthorized
			);

			assert_ok!(AccountTokenizer::request_redeem(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE
			));

			// Token is taken into the custodial of the module account,
			// and will not be burned until confirmed by the oracle.
			assert_eq!(ModuleNFT::owner(&0, &0), Some(TreasuryAccount::get()));
			assert_eq!(AccountTokenizer::minted_account(proxy.clone()), Some(0));

			// Original owner cannot transfer the NFT
			assert_noop!(
				ModuleNFT::transfer(Origin::signed(ALICE), BOB, (0, 0)),
				DispatchError::Module(ModuleError {
					index: 3,
					error: 4,
					message: Some("NoPermission"),
				},)
			);

			// Confirm the burn
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				1,
				vec![1],
				CALL_WEIGHT,
			));

			assert_eq!(ModuleNFT::owner(&0, &0), None);
			assert_eq!(AccountTokenizer::minted_account(proxy.clone()), None);

			let events = System::events();
			assert_eq!(
				events[events.len() - 2].event,
				Event::AccountTokenizer(crate::Event::AccountTokenRedeemed {
					account: proxy.clone(),
					token_id: 0,
					new_owner: ALICE,
				})
			);
			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 1,
					task_result: Ok(()),
				},
			));

			// XCM fee is burned.
			assert_eq!(Balances::free_balance(&ALICE), dollar(992));
			assert_eq!(Balances::reserved_balance(&ALICE), 0);
			assert_eq!(Balances::free_balance(&TREASURY), dollar(1));

			// cannot burn the same nft again
			assert_noop!(
				AccountTokenizer::request_redeem(Origin::signed(ALICE), proxy, ALICE),
				crate::Error::<Runtime>::AccountTokenNotFound
			);
		});
}

#[test]
fn can_remint_after_burn_token_nft() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000)), (BOB, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Mint the NFT.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));

			// Transfer the NFT
			assert_ok!(ModuleNFT::transfer(Origin::signed(ALICE), BOB, (0, 0)));

			// Burn the NFT
			assert_ok!(AccountTokenizer::request_redeem(
				Origin::signed(BOB),
				proxy.clone(),
				BOB
			));

			// Confirm the burn
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				1,
				vec![1],
				CALL_WEIGHT,
			));

			// Bob can now re-mint the account
			// Original owner of the proxy account must be passed in to verify.
			assert_noop!(
				AccountTokenizer::request_mint(Origin::signed(BOB), proxy.clone(), BOB.clone(), 1, 0, 0),
				DispatchError::Module(ModuleError {
					index: 6,
					error: 3,
					message: Some("FailedAnonymousProxyCheck",),
				},)
			);

			// Pass in original owner to mint
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(BOB),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				2,
				vec![1],
				CALL_WEIGHT,
			));

			// Transfer the NFT back to alice
			assert_ok!(ModuleNFT::transfer(Origin::signed(BOB), ALICE, (0, 1)));

			// Burn the NFT
			assert_ok!(AccountTokenizer::request_redeem(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE
			));

			// Confirm the burn
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				3,
				vec![],
				CALL_WEIGHT,
			));
		});
}

#[test]
fn cannot_double_mint() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			// Spawn a anonymous proxy account.
			assert_ok!(Proxy::anonymous(Origin::signed(ALICE), Default::default(), 0, 0u16,));
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Send 2 minting requests.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));

			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));

			// Accept the first mint.
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));
			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 0,
					task_result: Ok(()),
				},
			));

			// Once minted, the second request will be rejected
			let before_reject = Balances::free_balance(TreasuryAccount::get());
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				1,
				vec![1],
				CALL_WEIGHT,
			));

			// Request is rejected
			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 1,
					task_result: Ok(()),
				},
			));
			System::assert_has_event(Event::AccountTokenizer(crate::Event::MintRequestRejected {
				requester: ALICE,
			}));
			// Penalty is taken for double mint attempt
			assert_eq!(
				Balances::free_balance(TreasuryAccount::get()),
				before_reject + MintRequestDeposit::get()
			);

			// Transfer the NFT
			assert_ok!(ModuleNFT::transfer(Origin::signed(ALICE), BOB, (0, 0)));

			// Minting again will fail
			assert_noop!(
				AccountTokenizer::request_mint(Origin::signed(ALICE), proxy.clone(), ALICE.clone(), 1, 0, 0),
				DispatchError::Module(ModuleError {
					index: 6,
					error: 4,
					message: Some("AccountTokenAlreadyExists",),
				},)
			);
		});
}

#[test]
fn redeem_request_rejected() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Mint the NFT.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));

			// Governance can only transfer NFT owned by treasury
			assert_noop!(
				AccountTokenizer::return_custodial_account_token(Origin::signed(ORACLE), proxy.clone(), BOB),
				crate::Error::<Runtime>::CallerUnauthorized
			);

			assert_ok!(AccountTokenizer::request_redeem(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE
			));
			// Reject the burn
			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				1,
				vec![0],
				CALL_WEIGHT,
			));

			// Nft is given to Treasury Account
			assert_eq!(ModuleNFT::owner(&0, &0).unwrap(), TreasuryAccount::get());

			let events = System::events();
			assert_eq!(
				events[events.len() - 2].event,
				Event::AccountTokenizer(crate::Event::AccountTokenRedeemFailed { account: proxy.clone() })
			);
			System::assert_last_event(Event::ForeignStateOracle(
				module_foreign_state_oracle::Event::CallDispatched {
					query_id: 1,
					task_result: Ok(()),
				},
			));

			// Governance can return token back to Alice due to rejection
			assert_ok!(AccountTokenizer::return_custodial_account_token(
				Origin::signed(ORACLE),
				proxy.clone(),
				ALICE
			));
			// Nft is given to ALICE
			assert_eq!(ModuleNFT::owner(&0, &0).unwrap(), ALICE);
		});
}

#[test]
fn can_handle_when_request_mint_oracle_fails() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Mint the NFT.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));

			assert_eq!(Balances::free_balance(&ALICE), dollar(988));
			assert_eq!(Balances::reserved_balance(&ALICE), dollar(10));

			// can return user fund when oracle failed to respond to mint
			assert_ok!(AccountTokenizer::force_unreserve_funds(
				Origin::signed(ORACLE),
				ALICE,
				dollar(10),
				false,
			));

			assert_eq!(Balances::free_balance(&ALICE), dollar(998));
			assert_eq!(Balances::reserved_balance(&ALICE), 0);

			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));
			assert_eq!(Balances::free_balance(&ALICE), dollar(986));
			assert_eq!(Balances::reserved_balance(&ALICE), dollar(10));

			// can slash the reserved fund
			assert_ok!(AccountTokenizer::force_unreserve_funds(
				Origin::signed(ORACLE),
				ALICE,
				dollar(10),
				true,
			));

			assert_eq!(Balances::free_balance(&ALICE), dollar(986));
			assert_eq!(Balances::free_balance(&TREASURY), dollar(12));
			assert_eq!(Balances::reserved_balance(&ALICE), 0);

			// If slashed by mistake, the fund can be re-imbursed by the treasury
			assert_ok!(AccountTokenizer::transfer_treasury_funds(
				Origin::signed(ORACLE),
				ALICE,
				dollar(10)
			));
			assert_eq!(Balances::free_balance(&ALICE), dollar(996));
			assert_eq!(Balances::free_balance(&TREASURY), dollar(2));
		});
}

#[test]
fn can_remint_account_token() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Mint the NFT.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));

			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));
			// Burn the NFT directly through the NFT module
			assert_ok!(ModuleNFT::burn_from(&0, &0));

			assert_eq!(AccountTokenizer::minted_account(&proxy), Some(0));
			assert_eq!(ModuleNFT::owner(&0, &0), None);

			// Can remint burned NFT
			assert_ok!(AccountTokenizer::remint_burnt_nft(
				Origin::signed(ORACLE),
				proxy.clone(),
				ALICE
			));
			assert_eq!(AccountTokenizer::minted_account(&proxy), Some(1));
			assert_eq!(ModuleNFT::owner(&0, &1), Some(ALICE));

			System::assert_last_event(Event::AccountTokenizer(crate::Event::AccountTokenReminted {
				owner: ALICE,
				account: proxy.clone(),
				token_id: 1,
			}));
		});
}

#[test]
fn can_handle_oracle_failed_to_confirm_redeem() {
	ExtBuilder::default()
		.balances(vec![(ALICE, dollar(1_000))])
		.build()
		.execute_with(|| {
			AccountTokenizer::on_runtime_upgrade();
			let proxy = AccountId::new(hex!["7342619566cac76247062ffd59cd3fb3ffa3350dc6a5087938b9d1c46b286da3"]);

			// Mint the NFT.
			assert_ok!(AccountTokenizer::request_mint(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE.clone(),
				1,
				0,
				0
			));

			assert_ok!(ForeignStateOracle::respond_query_request(
				Origin::signed(ORACLE),
				0,
				vec![1],
				CALL_WEIGHT,
			));

			assert_ok!(AccountTokenizer::request_redeem(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE
			));

			// Token is taken into the custody of the treasury
			assert_eq!(AccountTokenizer::minted_account(&proxy), Some(0));
			assert_eq!(ModuleNFT::owner(&0, &0), Some(TREASURY));

			// can return NFT tokens held in custody
			assert_ok!(AccountTokenizer::return_custodial_account_token(
				Origin::signed(ORACLE),
				proxy.clone(),
				ALICE
			));

			assert_eq!(AccountTokenizer::minted_account(&proxy), Some(0));
			assert_eq!(ModuleNFT::owner(&0, &0), Some(ALICE));
			System::assert_last_event(Event::AccountTokenizer(crate::Event::CustodialAccountTokenReturned {
				token_id: 0,
				new_owner: ALICE,
			}));

			// Alternatively, the NFT can be burned
			assert_ok!(AccountTokenizer::request_redeem(
				Origin::signed(ALICE),
				proxy.clone(),
				ALICE
			));

			assert_ok!(AccountTokenizer::burn_nft(Origin::signed(ORACLE), proxy.clone()));
			assert_eq!(AccountTokenizer::minted_account(&proxy), None);
			assert_eq!(ModuleNFT::owner(&0, &0), None);
			System::assert_last_event(Event::ModuleNFT(module_nft::Event::BurnedToken {
				owner: TREASURY,
				class_id: 0,
				token_id: 0,
			}));
		});
}