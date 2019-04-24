use serde_derive::{Deserialize, Serialize};

pub mod input;
pub mod output;
pub mod runtime;
pub mod serial;

use input::*;
use output::*;

/// Holds information about the Game
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Info {
    name: String,
    step_interval: u32,
    players: Vec<Player>,
}

impl Info {
    /// Create a new structure of info about the game being played
    /// # Arguments
    /// * `name` - The title of the game
    /// * `steps_per_second` - The number of times Game::Step() should be called per second
    /// * `number_of_players` - How many players this games should have
    /// * `input` - The device type to use for each player
    pub fn new(
        name: &str,
        steps_per_second: i32,
        number_of_players: i32,
        input: InputDeviceType,
    ) -> Self {
        let player = Player { input };

        let mut players = Vec::with_capacity(number_of_players as usize);
        for _ in 0..number_of_players {
            players.push(player.clone());
        }

        Self {
            name: name.to_string(),
            step_interval: Self::steps_per_second_to_interval(steps_per_second),
            players,
        }
    }
    
    /// Gets the name of the game
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets time between steps of the game, in nanoseconds
    pub fn step_interval(&self) -> u32 {
        self.step_interval
    }

    /// Converts from steps per second to a time interval in nanoseconds
    pub fn steps_per_second_to_interval(steps: i32) -> u32 {
        1_000_000_000 / steps as u32
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Player {
    input: InputDeviceType,
}

/// The core trait used by Romy, games need to implement this. Romy will use these methods to run
/// the game.
pub trait Game {
    /// Simulates the game by one step
    /// 
    /// # Arguments
    /// * `arguments` - Info, such as inputs, to be used in this step
    fn step(&mut self, arguments: &StepArguments);

    /// Renders an image for Romy to display, can be called many times per step.
    ///
    /// This function can return any image.
    /// 
    /// # Arguments
    /// * `arguments` - Info, such as the width of the frame Romy is rendering too, to be used in
    /// this step
    fn draw(&self, arguments: &DrawArguments) -> Image;

    /// Renders some audio for Romy to play, called once per step.
    ///
    /// The sound returned currently needs to be at a sample rate of 44100hz, and have enough
    /// samples to cover the amount of time between calls to step.
    fn render_audio(&self, arguments: &RenderAudioArguments) -> Sound;
}

// Input Arguments /////////////////////////////////////////////////////////////////////////////////

/// Arguments passed for each step of the game
#[derive(Serialize, Deserialize, Default)]
pub struct StepArguments {
    input: InputArguments,
}

impl StepArguments {
    pub fn new(input: InputArguments) -> Self {
        Self { input }
    }

    /// Get the input for this step
    pub fn input(&self) -> &InputArguments {
        &self.input
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct InputArguments {
    players: Vec<Option<PlayerInputArguments>>,
}

impl InputArguments {
    pub fn new(players: Vec<Option<PlayerInputArguments>>) -> Self {
        Self { players }
    }

    /// Get the input for a specific player, will be None if there is no available player
    pub fn player(&self, player: i32) -> Option<&PlayerInputArguments> {
        let player = self.players.get(player as usize);

        if let Some(player) = player {
            return player.as_ref();
        }

        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlayerInputArguments {
    input: InputDevice,
}

impl PlayerInputArguments {
    /// Get the players NES style controller, will be None if there is no suitable input device, or
    /// one wasn't asked for in the supplied game info.
    pub fn nes(&self) -> Option<&Nes> {
        if let InputDevice::Nes(ref nes) = self.input {
            return Some(nes);
        }
        None
    }

    /// Get the players standard style controller, will be None if there is no suitable input device
    /// or one wasn't asked for in the supplied game info.
    pub fn controller(&self) -> Option<&Controller> {
        if let InputDevice::Controller(ref nes) = self.input {
            return Some(nes);
        }
        None
    }

    /// Get the players keyboard, will be None if there is no suitable input device or one wasn't 
    /// asked for in the supplied game info.
    pub fn keyboard(&self) -> Option<&Keyboard> {
        if let InputDevice::Keyboard(ref nes) = self.input {
            return Some(nes);
        }
        None
    }
}

// Draw arguments //////////////////////////////////////////////////////////////////////////////////

/// Arguments passed for each draw of the game
#[derive(Serialize, Deserialize, Debug)]
pub struct DrawArguments {
    width: i32,
    height: i32,
    step_offset: f32,
}

impl DrawArguments {
    pub fn new(width: i32, height: i32, step_offset: f32) -> Self {
        Self {
            width,
            height,
            step_offset,
        }
    }

    /// The horizontal width in pixels of the display area being used by Romy
    pub fn width(&self) -> i32 {
        self.width
    }

    /// The vertical height in pixels of the display area being used by Romy
    pub fn height(&self) -> i32 {
        self.height
    }

    /// The fraction of time since the last step call in a range of 0.0 - 1.0. 0.0 no time has
    /// passed, 0.5 = half way to the next step, 0.99 = almost all the way to the next step.
    pub fn step_offset(&self) -> f32 {
        self.step_offset
    }
}

// Render audio arguments //////////////////////////////////////////////////////////////////////////

/// Arguments passed for each audio render of the game
#[derive(Serialize, Deserialize, Debug)]
pub struct RenderAudioArguments {}
