use crate::pokemon::Pokemon;

#[derive(Debug, Clone, Copy, Default)]
pub struct SideConditions {
    pub stealth_rock: bool,
    pub spikes: u8,
    pub toxic_spikes: u8,
    pub sticky_web: bool,
    pub reflect: u8,
    pub light_screen: u8,
    pub tailwind: u8,
}

#[derive(Debug, Clone)]
pub struct Side {
    pub team: Vec<Pokemon>,
    pub active_index: usize,
    pub side_conditions: SideConditions,
    pub tera_used: bool,
}

impl Side {
    pub fn new(team: Vec<Pokemon>) -> Self {
        Self { team, active_index: 0, side_conditions: SideConditions::default(), tera_used: false }
    }

    pub fn active(&self) -> &Pokemon {
        &self.team[self.active_index]
    }

    pub fn active_mut(&mut self) -> &mut Pokemon {
        &mut self.team[self.active_index]
    }

    pub fn alive_count(&self) -> usize {
        self.team.iter().filter(|p| p.is_alive()).count()
    }

    pub fn has_alive_switch(&self) -> bool {
        self.team.iter().enumerate().any(|(i, p)| i != self.active_index && p.is_alive())
    }
}
