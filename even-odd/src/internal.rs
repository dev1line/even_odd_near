use crate::*;

#[near_bindgen]
impl EvenOddContract {
    pub(crate) fn internal_create_account(&mut self, account: AccountId) {
        let new_player_metadata = PlayerMetadata {
            bet_amount: U128(0),
            player: account.clone(),
            is_even: false,
        };
        self.players_array.push(account.clone());
        self.players.insert(&account, &new_player_metadata);
    }

    pub(crate) fn internal_set_default(&mut self) {
        for i in 0..self.players_array.len() {
            let new_player_metadata = PlayerMetadata {
                bet_amount: U128(0),
                player: self.players_array[i].clone(),
                is_even: false,
            };
            self.players
                .insert(&self.players_array[i].clone(), &new_player_metadata);
        }
    }
}
