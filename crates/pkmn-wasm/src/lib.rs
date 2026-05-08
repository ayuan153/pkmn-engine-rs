use wasm_bindgen::prelude::*;
use pkmn_engine::{Battle, Choice, BattleResult};

#[wasm_bindgen]
pub struct WasmBattle {
    inner: Battle,
}

#[wasm_bindgen]
impl WasmBattle {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> Self {
        let s = [
            (seed >> 48) as u16,
            (seed >> 32) as u16,
            (seed >> 16) as u16,
            seed as u16,
        ];
        Self { inner: Battle::default_test_battle(s) }
    }

    /// Get legal choices for a player as JSON array
    pub fn choices(&self, player: u8) -> JsValue {
        let choices = self.inner.choices(player);
        serde_wasm_bindgen::to_value(&choices).unwrap_or(JsValue::NULL)
    }

    /// Apply choices and return result ("ongoing", "win:0", "win:1", "tie")
    pub fn apply(&mut self, p1_choice: u8, p1_is_switch: bool, p2_choice: u8, p2_is_switch: bool) -> String {
        let c1 = if p1_is_switch { Choice::Switch(p1_choice) } else { Choice::Move(p1_choice) };
        let c2 = if p2_is_switch { Choice::Switch(p2_choice) } else { Choice::Move(p2_choice) };
        match self.inner.apply(c1, c2) {
            BattleResult::Ongoing => "ongoing".to_string(),
            BattleResult::Win(p) => format!("win:{}", p),
            BattleResult::Tie => "tie".to_string(),
        }
    }

    /// Get current turn number
    pub fn turn(&self) -> u16 { self.inner.turn }

    /// Get HP of active Pokemon for a player
    pub fn active_hp(&self, player: u8) -> u16 {
        self.inner.sides[player as usize].active().hp
    }

    /// Clone the battle state (for MCTS)
    pub fn clone_state(&self) -> WasmBattle {
        WasmBattle { inner: self.inner.clone() }
    }
}
