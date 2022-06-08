use near_sdk::{ext_contract, Gas, PromiseResult};

use crate::*;

pub const DEPOSIT_ONE_YOCTO: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;
pub const FT_TRANSFER_GAS: Gas = 10_000_000_000_000;
pub const FT_HARVEST_CALLBACK_GAS: Gas = 10_000_000_000_000;
pub const PRICE_PER_BET: Balance = 10_000_000_000_000_000_000_000; // 0.01 NEAR
pub const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_contract)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait ExtStakingContract {
    fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId);
    fn ft_withdraw_callback(&mut self, account_id: AccountId, old_account: AccountId);
}

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

// #[ext_contract[ext_cash_resolver]]
// pub trait TokenCashResolver {
//     fn ft_resolve_transfer(
//         &mut self,
//         sender_id: ValidAccountId,
//         receiver_id: ValidAccountId,
//         amount: U128,
//     ) -> U128;
// }
