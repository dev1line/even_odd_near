use crate::*;

#[near_bindgen]
impl EvenOddContract {
    pub fn get_owner(&self) -> AccountId {
        self.owner.clone()
    }

    pub fn get_ticket_address(&self) -> AccountId {
        self.ticket.clone()
    }

    pub fn get_cash_address(&self) -> AccountId {
        self.cash.clone()
    }
}
