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
const BALL_PROB_PER_SEC: f64 = 0.3;

const GROW_SPEED: f64 = 4.;
const SIZE_RATIO_TO_EAT: f64 = 1.2;

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub players: HashMap<usize, Player>,
    pub size: (f64, f64),
    pub balls: Vec<Ball>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ball {
    pub pos: (f64, f64),
    pub color: (u8, u8, u8),
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub pos: (f64, f64),
    pub direction: f64, // Radians
    pub speed: f64,
    pub size: f64,
    pub show_size: f64,
    pub color: (u8, u8, u8),
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

            player.show_size = (player.show_size - player.size) * (1. / GROW_SPEED).powf(dt) + player.size;

            let speed = player.speed / (player.size + 5.);

            let (dx, dy) = (speed * sin(player.direction), speed * cos(player.direction));
            player.pos.0 += dx * dt * 35.;
            player.pos.1 += dy * dt * 35.;

            if player.pos.0 < player.show_size {
                player.pos.0 = player.show_size;
            }
            if player.pos.1 < player.show_size {
                player.pos.1 = player.show_size;
            }
            if player.pos.0 > self.size.0 - player.show_size {
                player.pos.0 = self.size.0 - player.show_size;
            }
            if player.pos.1 > self.size.1 - player.show_size {
                player.pos.1 = self.size.1 - player.show_size;
            }

            self.balls.retain(|ball| {
                        let (dx, dy) = (ball.pos.0 - player.pos.0, ball.pos.1 - player.pos.1);
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < player.size - 1. {
                            player.size = ((player.size + 0.1).powi(2) + 3.).sqrt();
                            false
                        } else {
                            true
                        }
                    });
        }

        let mut old_players = mem::replace(&mut self.players, HashMap::new());

        // Suck in other players
        let mut succ: HashMap<usize, (f64, (f64, f64))> = HashMap::new(); // Id: (amount, to)

        for (id, player) in &old_players {
            for (oid, other) in &old_players {
                if oid == id { continue }

                if other.size < player.size / SIZE_RATIO_TO_EAT {
                    let (dx, dy) = (other.pos.0 - player.pos.0, other.pos.1 - player.pos.1);
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < player.size + other.size {
                        // ps = player.size, os = other.size
                        // ms = ps - os (Min succ, the lowest possible distance to not get eaten)
                        // Ms = ps + os (Max succ, the highest distance for the succ to have effect)
                        //
                        // We want to map ms to 1, Ms to 0 linearly.
                        // s = f(d) = k*d + m
                        // f(ms) = 1
                        // f(Ms) = 0
                        // k = Δs/Δd = 1/(ms-Ms)
                        // f(Ms) = k * Ms + m = 0
                        // Ms/(ms-Ms) + m = 1
                        // m = -Ms/(ms-Ms)
                        // f(d) = d / (ms-Ms) - Ms/(ms-Ms)
                        //      = d / (-2os) + ps / (2os) + 1/2

                        let succ_amount = dist / (-2. * other.size) + player.size / (2. * other.size) + 0.5;

                        succ.insert(*oid, (succ_amount, player.pos));
                    }
                }
            }
        }

        for (id, (amount, to)) in succ {
            if let Some(mut player) = old_players.get_mut(&id) {
                let (dx, dy) = (to.0 - player.pos.0, to.1 - player.pos.1);
                player.pos.0 += dx * dt * amount * 3.;
                player.pos.1 += dy * dt * amount * 3.;
            }
        }


        // Eat players
        let mut eaten_ids = HashSet::new();
        let mut size_adds: HashMap<usize, f64> = HashMap::new();
        for (id, player) in &old_players {

            for (oid, other) in &old_players {
                if oid == id { continue }
                if other.size < player.size / SIZE_RATIO_TO_EAT {
                    let (dx, dy) = (other.pos.0 - player.pos.0, other.pos.1 - player.pos.1);
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < player.size - other.size {
                        eaten_ids.insert(*oid);

                        let old_size = size_adds.get(id).unwrap_or(&0.).clone();

                        size_adds.insert(*id, old_size + other.size);
                    }
                }
            }
        }

        for (id, mut player) in old_players {
            // Area addition:
            //     r1 = player radius, r2 = other radius, r3 = resulting radius
            // a1 = pi * r1 ^ 2
            // a2 = pi * r2 ^ 2
            // a3 = a1 + a2 = pi * (r1 ^ 2 + r2 ^ 2)
            // a3 = pi * r3 ^ 2
            // r3 ^ 2 = r1 ^ 2 + r2 ^ 2
            // r3 = sqrt(r1 ^ 2 + r2 ^ 2)
            player.size = (player.size * player.size + size_adds.get(&id).map(|s| s * s).unwrap_or(0.)).sqrt();

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
            size: 3.,
            show_size: 0.,
            color: rng.gen::<(u8, u8, u8)>()
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

