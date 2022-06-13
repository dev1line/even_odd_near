use more_asserts::{assert_gt, assert_le};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::env::{current_account_id, predecessor_account_id, signer_account_id};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, setup_alloc, AccountId, Balance, PanicOnDefault, Promise,
    PromiseOrValue, PromiseResult,
};
use rand::Rng;
use std::convert::TryInto;

pub mod core_impl;
pub mod enumeration;
pub mod internal;
pub mod util;

use crate::core_impl::*;
use crate::enumeration::*;
use crate::internal::*;
use crate::util::*;

setup_alloc!();

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct PlayerMetadata {
    bet_amount: U128,
    player: AccountId,
    is_even: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct EvenOddContract {
    pub owner: AccountId,
    pub cash: AccountId,
    pub ticket: AccountId,
    players_array: Vec<AccountId>,
    players: UnorderedMap<AccountId, PlayerMetadata>,
    total_bet_amount: u128,
    total_bet_amount_per_roll: u128,
    roll_id: u32,
}

#[near_bindgen]
impl EvenOddContract {
    #[init]
    pub fn constructor(_dealer: AccountId, _cash: AccountId, _ticket: AccountId) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner: signer_account_id(),
            cash: _cash,
            ticket: _ticket,
            players_array: Vec::new(),
            players: UnorderedMap::new(b"players".to_vec()),
            total_bet_amount: 0,
            total_bet_amount_per_roll: 0,
            roll_id: 1,
        }
    }

    // deposit for paid fee create account info to store in blockkchain contract
    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        assert_at_least_one_yocto();
        let account = account_id.unwrap_or_else(|| predecessor_account_id());

        let player_metadata: Option<PlayerMetadata> = self.players.get(&account);

        if player_metadata.is_some() {
            refund_deposit(0);
        } else {
            let before_storage_usage = env::storage_usage();
            self.internal_create_account(account.clone());
            let after_storage_usage = env::storage_usage();

            refund_deposit(after_storage_usage - before_storage_usage);
        }
    }
    #[payable]
    pub fn withdraw(&mut self, amount: U128) {
        assert_gt!(u128::from(amount), 0, "Amount must be not zero!");
        assert_le!(
            amount.0,
            self.get_dealer_balance().0,
            "Amount exceeds balance"
        );

        ext_ft_contract::ft_transfer(
            predecessor_account_id(),
            amount,
            Some(String::from("Withdraw !")),
            self.cash.clone(),
            amount.0,
            FT_TRANSFER_GAS,
        );

        env::log(
            format!(
                " Withdraw from {} to {} with amount is {} ",
                String::from(current_account_id()),
                String::from(predecessor_account_id()),
                u128::from(amount)
            )
            .as_bytes(),
        );
    }
    #[private]
    pub fn is_have_ticket(&self) -> bool {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Contract expected a result on the callback"
        );
        match env::promise_result(0) {
            PromiseResult::Successful(_value) => {
                let value = near_sdk::serde_json::from_slice::<U128>(&_value).unwrap();
                if value.0 > 0 {
                    return true;
                };
                return false;
            }
            _ => false,
        }
    }
    #[private]
    pub fn have(&self) -> U128 {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Contract expected a result on the callback"
        );
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
                if balance.0 > 0 {
                    return balance;
                } else {
                    return U128(0);
                }
            }
            _ => U128(0),
        }
    }
    pub fn nft_balance_of(&self) -> bool {
        ext_nft_contract::nft_supply_for_owner(
            predecessor_account_id(),
            self.ticket.clone(),
            NO_DEPOSIT,
            FT_CALL_GAS,
        );

        ext_self::nft_balance_callback(current_account_id(), NO_DEPOSIT, FT_CALL_GAS);
        self.is_have_ticket()
    }
    pub fn bet(&mut self, is_even: bool, amount: U128) {
        let balance = self.nft_balance_of();
        assert!(balance, "You need to buy a ticket to play this game");

        // assert_le!(env::block_timestamp(), self.ticket.get_expired_time(token_id), "Your ticket is expired");
        assert!(
            !self.is_already_bet(predecessor_account_id()),
            "Already bet"
        );

        let new_player_metadata = PlayerMetadata {
            bet_amount: amount,
            player: predecessor_account_id(),
            is_even,
        };
        self.players_array.push(predecessor_account_id());
        self.players
            .insert(&predecessor_account_id(), &new_player_metadata);
        self.total_bet_amount += amount.0;
        self.total_bet_amount_per_roll += amount.0;
    }

    pub fn roll_dice(&mut self) {
        assert_at_least_one_yocto();
        assert_only_owner_access(self.owner.clone());
        assert_gt!(self.total_bet_amount_per_roll, 0, "No one place bet");

        let dice_number_1 = self.generate_random_number();
        let dice_number_2 = self.generate_random_number();

        let is_even = (&dice_number_1 + &dice_number_2) % 2 == 0;

        for i in 0..self.players_array.len() {
            let player_metadata = self.players.get(&self.players_array[i]).unwrap();
            if player_metadata.is_even == is_even && player_metadata.bet_amount.0 > 0 {
                self.transfer_cash(
                    player_metadata.player,
                    U128::from(
                        u128::from(self.players.get(&self.players_array[i]).unwrap().bet_amount)
                            * 2,
                    ),
                );
                // Allow admin can bet and control flow
            }
        }
        self.reset_board();
        self.roll_id = self.roll_id + 1;

        env::log(
            format!(
                " Roll id: {}, Roll dice: {} - {} with result is {}",
                self.roll_id, &dice_number_1, &dice_number_2, is_even
            )
            .as_bytes(),
        );
    }

    pub fn is_already_bet(&self, account: AccountId) -> bool {
        if u128::from(self.players.get(&account).unwrap().bet_amount) > 0 {
            return true;
        }
        false
    }

    pub fn get_dealer_balance(&self) -> U128 {
        ext_ft_contract::ft_balance_of(
            self.owner.clone(),
            self.cash.clone(),
            NO_DEPOSIT,
            FT_TRANSFER_GAS,
        )
        .then(ext_self::ft_balance_callback(
            current_account_id(),
            NO_DEPOSIT,
            FT_CALL_GAS,
        ));
        self.have()
    }

    pub fn get_bet_amount_of(&self, account: AccountId) -> U128 {
        self.players.get(&account).unwrap().bet_amount
    }

    pub fn get_player_info(&self, account: AccountId) -> (U128, bool) {
        let bet_amount = self.players.get(&account).unwrap().bet_amount;
        let is_even = self.players.get(&account).unwrap().is_even;
        (bet_amount, is_even)
    }

    #[private]
    pub fn transfer_cash(&mut self, account: AccountId, amount: U128) {
        ext_ft_contract::ft_transfer(
            account.clone(),
            amount,
            Some(String::from("Pay reward for players !")),
            self.cash.clone(),
            amount.0,
            FT_TRANSFER_GAS,
        )
        .then(ext_self::ft_pay_reward_callback(
            account.clone(),
            amount,
            current_account_id(),
            NO_DEPOSIT,
            PAY_REWARD_CALLBACK_GAS,
        ));
    }

    #[private]
    pub fn reset_board(&mut self) {
        self.total_bet_amount_per_roll = 0;
        self.internal_set_default();
    }

    #[private]
    pub fn generate_random_number(&self) -> usize {
        rand::thread_rng().gen_range(0..100)
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::AccountId;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn check_constructor() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = EvenOddContract::constructor(accounts(0), accounts(1), accounts(2));
        println!(
            "{:?}, {:?}, {:?}",
            contract.get_owner(),
            contract.get_cash_address(),
            contract.get_ticket_address()
        );
        assert_eq!(contract.get_owner(), accounts(0));
        assert_eq!(contract.get_cash_address(), accounts(1));
        assert_eq!(contract.get_ticket_address(), accounts(2));
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(3));
        testing_env!(context.build());
        let contract = EvenOddContract::default();
    }
}
