use near_sdk::serde_json::{from_slice, from_str, Value};
use near_sdk::Gas;

use crate::*;
pub const FT_TRANSFER_GAS: Gas = Gas(10_000_000_000_000);
pub const FT_CALL_GAS: Gas = Gas(10_000_000_000_000);
pub const WITHDRAW_CALLBACK_GAS: Gas = Gas(10_000_000_000_000);
pub const PAY_REWARD_CALLBACK_GAS: Gas = Gas(10_000_000_000_000);
pub const NO_DEPOSIT: Balance = 0;
pub const DEPOSIT_ONE_YOCTOR: Balance = 1;

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_contract)]
pub trait FungibleTokenCore {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
        memo: Option<String>,
    ) -> Promise;
    fn ft_total_supply(&self) -> U128;
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

#[ext_contract(ext_nft_contract)]
pub trait NonFungibleTokenCore {
    fn nft_total_supply(&self) -> U128; // Lấy tổng số token đang có trong contract
    fn nft_supply_for_owner(&self, account_id: AccountId) -> U128; // Lấy tổng số token đang có của account_id
    fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>); // Lấy danh sách token có paging
    fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ); // Lấy danh sach token của account_id
}

#[ext_contract(ext_self)]
pub trait ExtStakingContract {
    fn ft_transfer_callback(&mut self, account_id: AccountId, amount: U128, is_even: bool);
    fn ft_pay_reward_callback(&mut self, account_id: AccountId, amount: U128);
    fn ft_balance_callback(&mut self);
    fn nft_balance_callback(&mut self);
}
macro_rules! enum_str {
    (enum $name:ident {
        $($variant:ident = $val:expr),*,
    }) => {
        enum $name {
            $($variant = $val),*
        }

        impl $name {
            fn name(&self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),*
                }
            }
        }
    };
}

enum_str! {
    enum Action {
        // StorageDeposit = "deposit".to_string(),
        Bet = 0x00,
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for EvenOddContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // Handle when have transfer into contract
        let msg_obj: Value = from_str(&msg).unwrap();
        //msg_obj = "{\"action\":\"deposit\",\"params\":{\"amount\":10}}"; NOT NEED
        //msg_obj = "{\"action\":\"bet\",\"params\":{\"is_even\":\"true\",\"amount\":10}}";
        let action = msg_obj.get("action").unwrap().as_str();
        let params_obj = msg_obj.get("params").unwrap();
        let _is_even = params_obj.get("is_even").unwrap().as_bool().unwrap();
        let _amount = params_obj.get("amount").unwrap().as_u64().unwrap();
        let Key = Action::Bet.name().to_string();
        match action {
            Key => self.bet(_is_even, U128(_amount.into())),
            _ => panic!("ERR_CALL_FUNCTION !"),
        }
        // return amount not used
        PromiseOrValue::Value(U128(0))
    }
}

impl EvenOddContract {
    pub fn ft_transfer_callback(
        &mut self,
        account_id: AccountId,
        amount: U128,
        is_even: bool,
    ) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                let value = PlayerMetadata {
                    bet_amount: amount,
                    is_even,
                    player: account_id.clone(),
                };
                self.players.insert(&account_id, &value);

                self.total_bet_amount += amount.0;
                self.total_bet_amount_per_roll += amount.0;
                amount
            }
            PromiseResult::Failed => env::panic(b"ERR_CALL_FAILED"),
        }
    }

    pub fn ft_pay_reward_callback(&mut self, account_id: AccountId, amount: U128) {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => {
                // Update new data
                env::log(
                    format!(" Pay reward for {} with {} tokens", account_id, amount.0).as_bytes(),
                );
            }
            PromiseResult::Failed => {
                // Handle rollback data
                env::panic(b"ERR_CALL_FAILED");
            }
        }
    }

    pub fn ft_balance_callback(&mut self) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            // PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => U128(0),
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

    pub fn nft_balance_callback(&mut self) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULTS");
        match env::promise_result(0) {
            // PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => U128(0),
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
}
