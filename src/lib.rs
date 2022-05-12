use near_contract_standards::fungible_token::resolver::FungibleTokenResolver;
use std::convert::TryInto;
// use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
// use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, init, near_bindgen, setup_alloc, AccountId, Balance, PanicOnDefault};
pub mod ticket;
pub mod token;
use ticket::Ticket;
use token::Token;
setup_alloc!();
const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
// pub trait TicketInterface {
//     /// Returns true if token should be returned to `sender_id`
//     pub fn nft_mint(
//         &mut self,
//         token_id: TokenId,
//         receiver_id: ValidAccountId,
//         token_metadata: TokenMetadata,
//     ) -> Token;
// }
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[serde(crate = "near_sdk::serde")]
pub struct PlayerMetadata {
    bet_amount: usize,
    player: AccountId,
    is_even: bool,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct EvenOddContract {
    owner: AccountId,
    cash: Token,
    ticket: Ticket,
    players_array: Vec<AccountId>,
    players: UnorderedMap<AccountId, PlayerMetadata>,
    total_bet_amount: usize,
    total_bet_amount_per_roll: usize,
    roll_id: usize,
}

#[near_bindgen]
impl EvenOddContract {
    #[init]
    pub fn constructor(dealer: AccountId) -> Self {
        use near_sdk::env::signer_account_id;
        Self {
            owner: dealer,
            cash: Token::new_default_meta(
                signer_account_id().try_into().unwrap(),
                TOTAL_SUPPLY.into(),
            ),
            ticket: Ticket::new_default_meta(signer_account_id().try_into().unwrap()),
            players_array: Vec::new(),
            players: UnorderedMap::new(b"players".to_vec()),
            total_bet_amount: 0,
            total_bet_amount_per_roll: 0,
            roll_id: 1,
        }
    }
    #[payable]
    pub fn transfer(&mut self, amount: U128) {
        // assert!()
        use near_sdk::env::{current_account_id, predecessor_account_id};
        self.cash.ft_resolve_transfer(
            predecessor_account_id().try_into().unwrap(),
            current_account_id().try_into().unwrap(),
            amount,
        );
    }
    pub fn withdraw() {}
    pub fn bet() {}
    pub fn roll_dice() {}
    pub fn is_already_bet() {}
    pub fn get_dealer_balance() {}
    pub fn get_bet_amount_of() {}
    pub fn get_player_info() {}
    pub fn transfer_token() {}
    pub fn reset_board() {}
    pub fn generate_random_number() {}
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
