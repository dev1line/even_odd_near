use more_asserts::{assert_le, assert_gt};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
// use near_contract_standards::non_fungible_token::enumeration::NonFungibleTokenEnumeration;
use near_sdk::env::{current_account_id, predecessor_account_id};
use std::convert::TryInto;
// use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
// use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap};

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, setup_alloc, AccountId, Balance, PanicOnDefault, ext_contract};
use rand::Rng;

pub mod ticket;
pub mod token;
use ticket::Ticket;
use token::Token;


setup_alloc!();

const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

// #[ext_contract[ext_cash_resolver]]
// pub trait TokenCashResolver {
//     fn ft_resolve_transfer(
//         &mut self,
//         sender_id: ValidAccountId,
//         receiver_id: ValidAccountId,
//         amount: U128,
//     ) -> U128;
// }
#[ext_contract[ext_ticket]]
pub trait TokenTicket {
    fn nft_total_supply(self) -> U128;
    fn nft_tokens(
        &self,
        from_index: Option<U128>, // default: "0"
        limit: Option<u64>,       // default: unlimited (could fail due to gas limit)
    ) -> Vec<Token>;
    fn nft_supply_for_owner(self, account_id: ValidAccountId) -> U128;
    fn nft_tokens_for_owner(
        &self,
        account_id: ValidAccountId,
        from_index: Option<U128>, // default: "0"
        limit: Option<u64>,       // default: unlimited (could fail due to gas limit)
    ) -> Vec<Token>;
}
#[ext_contract[ext_cash]]
pub trait TokenCash {
    fn ft_transfer(&mut self, receiver_id: ValidAccountId, amount: U128, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: ValidAccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128>;
    fn ft_total_supply(&self) -> U128;
    fn ft_balance_of(&self, account_id: ValidAccountId) -> U128;
    fn ft_resolve_transfer(
        &mut self,
        sender_id: ValidAccountId,
        receiver_id: ValidAccountId,
        amount: U128,
    ) -> U128;
}


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
    owner: ValidAccountId,
    cash: Token,
    ticket: AccountId,
    players_array: Vec<AccountId>,
    players: UnorderedMap<AccountId, PlayerMetadata>,
    total_bet_amount: usize,
    total_bet_amount_per_roll: usize,
    roll_id: usize,
}

#[near_bindgen]
impl EvenOddContract {
    #[init]
    pub fn constructor(_dealer: AccountId, _cash: AccountId, _ticket: AccountId) -> Self {
        use near_sdk::env::signer_account_id;
        assert!(!env::state_exists(), "The contract is already initialized");
        Self {
            owner: signer_account_id().try_into().unwrap(),
            cash: Token::new_default_meta(signer_account_id().try_into().unwrap(),TOTAL_SUPPLY.into()),
            ticket: _ticket,//Ticket::new_default_meta(signer_account_id().try_into().unwrap())
            players_array: Vec::new(),
            players: UnorderedMap::new(b"players".to_vec()),
            total_bet_amount: 0,
            total_bet_amount_per_roll: 0,
            roll_id: 1,
        }
    }

    #[payable]
    pub fn transfer(&mut self, amount: U128) {
        use near_sdk::env::{current_account_id, predecessor_account_id};
        self.cash.ft_resolve_transfer(
            predecessor_account_id().try_into().unwrap(),
            current_account_id().try_into().unwrap(),
            amount,
        );
    }

    pub fn withdraw(&mut self, amount: U128) {
        use near_sdk::env::{current_account_id, predecessor_account_id};
        assert_gt!(u128::from(amount), 0, "Amount must be not zero!");
        assert_le!(u128::from(amount), u128::from(self.get_dealer_balance()), "Amount exceeds balance");
        self.cash.ft_resolve_transfer(
            current_account_id().try_into().unwrap(), 
            predecessor_account_id().try_into().unwrap(), 
            amount
        );

        env::log(
            format!(" Withdraw from {} to {} with amount is {} ", 
                String::from(current_account_id()), 
                String::from(predecessor_account_id()),
                u128::from(amount)
            ).as_bytes()
        );
    }

    pub fn bet(&self, is_even: bool, amount: U128) {
        // let account_balance = env::account_balance();
        // let token_id = 
        let balance = ext_ticket::nft_supply_for_owner(predecessor_account_id().try_into().unwrap(),&current_account_id().try_into().unwrap(),0,25_000_000_000_000);
        assert_gt!(balance, 0, "You need to buy a ticket to play this game");
        // assert_le!(env::block_timestamp(), self.ticket.get_expired_time(token_id), "Your ticket is expired");
        assert!(!self.is_already_bet(predecessor_account_id()) == true, "Already bet");

    }
    pub fn roll_dice(&mut self) {
        assert_eq!(self.owner, env::predecessor_account_id().try_into().unwrap(), "Only owner can call roll dice");
        assert_gt!(self.total_bet_amount_per_roll, 0, "No one place bet");

        let dice_number_1 = self.generate_random_number();
        let dice_number_2 = self.generate_random_number();

        let is_even = (dice_number_1 + dice_number_2) %2 == 0;

        env::log(
            format!(" Roll id: {}, Roll dice: {} - {} with resul is {}", 
              self.roll_id, 
              dice_number_1,
              dice_number_2,
                is_even
            ).as_bytes()
        );
        for i in 0..self.players_array.len() {
            if self.players.get(&self.players_array[i]).unwrap().is_even == is_even {
                self.transfer_money(
                    self.players_array[i].clone(), 
                    U128::from(
                        u128::from(
                            self.players.get(
                                &self.players_array[i]
                            ).unwrap().bet_amount
                        )*2
                    )
                );
            }
        }
        self.reset_board();
        self.roll_id = self.roll_id + 1;
    }
    pub fn is_already_bet(&self, account: AccountId) -> bool {
        if u128::from(self.players.get(&account).unwrap().bet_amount) > 0 {
            return true;
        }
        false
    }
    pub fn get_dealer_balance(&self) -> U128 {
        self.cash.ft_balance_of(self.owner.clone())
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
    pub fn transfer_money(&mut self, account: AccountId, amount: U128) {
       
        if account.clone().eq(&String::from(self.owner.clone())) {
            self.cash.ft_resolve_transfer(
                current_account_id().try_into().unwrap(), 
                predecessor_account_id().try_into().unwrap(), 
                amount
            );
        } else {
            self.cash.ft_resolve_transfer(
                current_account_id().try_into().unwrap(), 
                self.players.get(&account).unwrap().player.try_into().unwrap(), 
                amount
            );
        }
    }

    #[private]
    pub fn reset_board(&mut self) {
        self.players_array = vec![];
        self.total_bet_amount_per_roll = 0;
    }

    #[private]
    pub fn generate_random_number(&self) -> usize {
        rand::thread_rng().gen_range(0..100)
    }
}

// #[cfg(all(test, not(target_arch = "wasm32")))]
// mod tests {
//     use super::*;
//     use near_sdk::MockedBlockchain;
//     use near_sdk::json_types::ValidAccountId;
//     use near_sdk::{testing_env};
//     use near_sdk::test_utils::{accounts, VMContextBuilder};

//     fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
//         let mut builder = VMContextBuilder::new();
//         builder
//             .current_account_id(accounts(0))
//             .signer_account_id(predecessor_account_id.clone())
//             .predecessor_account_id(predecessor_account_id);
//         builder
//     }

//     // #[test]
//     // fn check_update_post() {
//     //     let mut context = get_context(accounts(0));
//     //     testing_env!(context.build());
//     //     let mut contract = Article::default();
//     //     contract.create_post("howdy".to_string());
//     //     contract.create_post("sang".to_string());

//     //     assert_eq!(
//     //         "sang".to_string(),
//     //         contract.get_value(1)
//     //     );
//     //     let post = Post { id: 1, message: "sangdeptrai".to_owned(), author: env::current_account_id() };
//     //     contract.update_post(1, &post);
//     //     assert_eq!(
//     //         "sangdeptrai".to_string(),
//     //         contract.get_value(1)
//     //     );

//     //     testing_env!(context
//     //         .storage_usage(env::storage_usage())
//     //         .predecessor_account_id(accounts(0))
//     //         .build());
//     //     let is_del = contract.delete_post(1);

//     //     assert_eq!(
//     //         is_del,
//     //      true
//     //     );
//     // }

// }
