//! This module contains code that is commonly needed by runtime implementations for various
//! platforms, its not intended to be used for other purposes. 
 
use super::*;

/// A version of the Game trait with mutable draw/render_audio. Some implementations need this.
pub trait GameMut {
    fn step(&mut self, arguments: &StepArguments);
    fn draw(&mut self, arguments: &DrawArguments) -> Image;
    fn render_audio(&mut self, arguments: &RenderAudioArguments) -> Sound;
}

/// A wrapper to convert a immutable Game to a mutable one
pub struct GameMutMap {
    game: Box<Game>,
}

impl GameMutMap {
    pub fn new(game: Box<Game>) -> Self {
        Self { game }
    }
}

impl GameMut for GameMutMap {
    fn step(&mut self, arguments: &StepArguments) {
        self.game.step(arguments)
    }
    fn draw(&mut self, arguments: &DrawArguments) -> Image {
        self.game.draw(arguments)
    }
    fn render_audio(&mut self, arguments: &RenderAudioArguments) -> Sound {
        self.game.render_audio(arguments)
    }
}

/// A structure for holding a game and its info struct together
pub struct RunBundle {
    pub game: Box<GameMut>,
    pub info: Info,
}

impl RunBundle {
    pub fn new(game: Box<GameMut>, info: Info) -> Self {
        Self { game, info }
    }
}
