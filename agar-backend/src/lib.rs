#[macro_use]
extern crate serde_derive;

extern crate serde;

mod math;
use math::*;

use std::collections::HashMap;


#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub players: HashMap<usize, Player>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Player {
    pub pos: (f64, f64),
    pub direction: f64, // Radians
    pub speed: f64,
    pub size: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IdPlayerCommand {
    pub id: usize,
    pub command: PlayerCommand
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PlayerCommand {
    SetDirectionAndSpeed(f64, f64)
}

#[derive(Serialize, Deserialize, Debug)]
pub enum SynchrosizationMessage {
    Command(IdPlayerCommand),
    SyncState(State)
}

impl State {
    pub fn new() -> State {
        State { players: HashMap::new() }
    }

    pub fn tick(&mut self, dt: f64) {
        for (_id, player) in self.players.iter_mut() {
            let speed = player.speed.max(20.) / player.size;

            let (dx, dy) = (speed * sin(player.direction), speed * cos(player.direction));
            player.pos.0 += dx * dt;
            player.pos.1 += dy * dt;
        }
    }

    pub fn do_command(&mut self, command: IdPlayerCommand) {
        if let Some(player) = self.players.get_mut(&command.id) {
            match command.command {
                PlayerCommand::SetDirectionAndSpeed(dir, speed) => {
                    player.direction = dir;
                    player.speed = speed;
                }
            }
        }
    }
}

