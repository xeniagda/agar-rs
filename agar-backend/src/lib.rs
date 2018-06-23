#[macro_use]
extern crate serde_derive;

extern crate serde;

#[cfg(feature="server-side")]
extern crate rand;


mod math;
use math::*;

use std::collections::{HashMap, HashSet};
use std::mem;

#[cfg(feature = "server-side")]
const BALL_PROB_PER_SEC: f64 = 0.1;


#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub players: HashMap<usize, Player>,
    pub size: (f64, f64),
    pub balls: Vec<Ball>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ball {
    pub pos: (f64, f64),
    pub color: (u8, u8, u8)
}


#[derive(Serialize, Deserialize, Debug, Clone)]
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
        State {
            players: HashMap::new(),
            balls: vec![],
            size: (300., 300.),
        }
    }

    pub fn tick(&mut self, dt: f64) {
        for (_id, player) in self.players.iter_mut() {
            let speed = player.speed / (player.size + 5.);

            let (dx, dy) = (speed * sin(player.direction), speed * cos(player.direction));
            player.pos.0 += dx * dt * 35.;
            player.pos.1 += dy * dt * 35.;

            if player.pos.0 < player.size {
                player.pos.0 = player.size;
            }
            if player.pos.1 < player.size {
                player.pos.1 = player.size;
            }
            if player.pos.0 > self.size.0 - player.size {
                player.pos.0 = self.size.0 - player.size;
            }
            if player.pos.1 > self.size.1 - player.size {
                player.pos.1 = self.size.1 - player.size;
            }

            self.balls.retain(|ball| {
                        let (dx, dy) = (ball.pos.0 - player.pos.0, ball.pos.1 - player.pos.1);
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < player.size - 1 {
                            player.size += 1.;
                            false
                        } else {
                            true
                        }
                    });
        }

        let old_players = mem::replace(&mut self.players, HashMap::new());

        let mut eaten_ids = HashSet::new();
        let mut size_adds: HashMap<usize, f64> = HashMap::new();
        for (id, player) in &old_players {

            for (oid, other) in &old_players {
                if oid == id { continue }
                if other.size >= player.size { continue }

                let (dx, dy) = (other.pos.0 - player.pos.0, other.pos.1 - player.pos.1);
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < player.size {
                    eaten_ids.insert(*oid);

                    let old_size = size_adds.get(id).unwrap_or(&0.).clone();

                    size_adds.insert(*id, old_size + other.size);
                }
            }
        }

        for (id, mut player) in old_players {
            player.size += *size_adds.get(&id).unwrap_or(&0.);

            if !eaten_ids.contains(&id) {
                self.players.insert(id, player);
            }
        }

        #[cfg(feature = "server-side")]
        self.do_server_side_stuff(dt);
    }

    #[cfg(feature = "server-side")]
    pub fn do_server_side_stuff(&mut self, dt: f64) {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();

        // We want rand() < x repeated 1/dt times be true with probability BALL_PROB_PER_SEC.
        // The probability of rand() < x is x, so
        // 1-(1-x)^(1/dt) = BALL_PROB_PER_SEC
        // (1-x)^(1/dt) = 1 - BALL_PROB_PER_SEC
        // 1-x = (1 - BALL_PROB_PER_SEC)^dt
        // x = 1 - (1 - BALL_PROB_PER_SEC) ^ dt
        if rng.gen::<f64>() < 1. - (1. - BALL_PROB_PER_SEC).powf(dt) {
            // Add ball
            self.balls.push(
                Ball {
                    pos: ( rng.gen_range(1., self.size.0 - 1.), rng.gen_range(1., self.size.1 - 1.) ),
                    color: rng.gen::<(u8, u8, u8)>()
                });
        }
    }

    #[cfg(feature = "server-side")]
    pub fn add_player(&mut self, id: usize) {
        use rand::{thread_rng, Rng};

        let mut rng = thread_rng();

        let player = Player {
            pos: ( rng.gen_range(0., self.size.0), rng.gen_range(0., self.size.1) ),
            direction: 0.,
            speed: 0.,
            size: 2.
        };

        self.players.insert(id, player);

    }

    pub fn do_command(&mut self, command: IdPlayerCommand) {
        if let Some(player) = self.players.get_mut(&command.id) {
            match command.command {
                PlayerCommand::SetDirectionAndSpeed(dir, speed) => {
                    player.direction = dir;
                    player.speed = speed.max(1.);
                }
            }
        }
    }
}

