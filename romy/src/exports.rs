use super::*;
use lazy_static::lazy_static;
use mut_static::MutStatic;
use romy_core::serial::*;
use std::collections::HashMap;

/// Exports the api version, in the case of breaking api changes the runtime should be able to adapt
#[no_mangle]
extern "C" fn romy_api_version() -> i32 {
    1
}

/// Allocate some WASM accessible memory for use by the runtime
///
/// # Arguments
/// * `size` - The number of bytes to allocate.
///
/// Returns a pointer to the allocated memory
#[no_mangle]
extern "C" fn allocate(size: i32) -> *mut u8 {
    let mut game = DATA.write().unwrap();
    game.allocate(size)
}

/// Deallocate memory previously allocated by allocate()
///
/// # Arguments
/// * `pointer` - The number of bytes to allocate.
#[no_mangle]
extern "C" fn deallocate(pointer: *const u8) {
    let mut game = DATA.write().unwrap();
    game.deallocate(pointer)
}

/// Steps the game forward
///
/// # Arguments
/// * `pointer` - A pointer to a romy::StepArguments structure encoded via 
/// romy_core::serial::encode_with_size
#[no_mangle]
extern "C" fn step(pointer: *const u8) {
    let game = unsafe { &mut ROOT };
    game.step(pointer)
}

/// Renders an image of the game
///
/// # Arguments
/// * `pointer` - A pointer to a romy::DrawArguments structure encoded via 
/// romy_core::serial::encode_with_size
/// 
/// Returns a romy::Image encoded with romy_core::serial::encode_with_size
#[no_mangle]
extern "C" fn draw(pointer: *const u8) -> *const u8 {
    let game = unsafe { &mut ROOT };
    game.draw(pointer)
}

/// Renders a steps worth of audio of the game
///
/// # Arguments
/// * `pointer` - A pointer to a romy::RenderAudioArguments structure encoded via 
/// romy_core::serial::encode_with_size
/// 
/// Returns a romy::Sound encoded with romy_core::serial::encode_with_size
#[no_mangle]
extern "C" fn render_audio(pointer: *const u8) -> *const u8 {
    let game = unsafe { &mut ROOT };
    game.render_audio(pointer)
}

lazy_static! {
    static ref DATA: MutStatic<MemoryAllocator> = { MutStatic::from(MemoryAllocator::new()) };
}

/// Moves ownership of some data to the host runtime, will need to be freed with
/// exports::deallocate()
pub fn move_ownership_to_host(object: impl serde::Serialize) -> *mut u8 {
    let mut game = DATA.write().unwrap();
    let encoded = encode_with_size(&object);
    let alloc = game.allocate(encoded.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(encoded.as_ptr(), alloc, encoded.len());
    }

    alloc
}

/// Structure for keeping track of memory allocated by the host
struct MemoryAllocator {
    next_id: i32,
    id_map: HashMap<usize, i32>,
    external_memory: HashMap<i32, Vec<u8>>,
}

impl MemoryAllocator {
    fn new() -> Self {
        Self {
            next_id: 0,
            id_map: HashMap::new(),
            external_memory: HashMap::new(),
        }
    }
    fn allocate(&mut self, size: i32) -> *mut u8 {
        let memory = vec![0; size as usize];
        let id = self.next_id;
        self.next_id += 1;
        self.external_memory.insert(id, memory);
        let result = self.external_memory.get_mut(&id).unwrap().as_mut_ptr();
        self.id_map.insert(result as usize, id);
        result
    }
    fn deallocate(&mut self, pointer: *const u8) {
        let id = pointer as usize;
        self.external_memory.remove(&self.id_map[&id]).unwrap();
        self.id_map.remove(&id).unwrap();
    }
}

pub static mut ROOT: Root = Root { game: None };

/// Used as a connection from exported functions to a Game
pub struct Root {
    game: Option<Box<Game>>,
}

impl Root {
    /// Make the connection
    pub fn connect(&mut self, game: Box<super::Game>) {
        self.game = Some(game);
    }
    fn step(&mut self, pointer: *const u8) {
        let step_input: StepArguments = unsafe { decode_with_size_ptr(pointer) };

        if let Some(app) = &mut self.game {
            app.step(&step_input);
            return;
        }

        panic!();
    }
    fn draw(&mut self, pointer: *const u8) -> *const u8 {
        let draw_input: DrawArguments = unsafe { decode_with_size_ptr(pointer) };

        if let Some(app) = &mut self.game {
            let image = app.draw(&draw_input);
            return move_ownership_to_host(image);
        }

        panic!();
    }
    fn render_audio(&mut self, pointer: *const u8) -> *const u8 {
        let render_audio_input: RenderAudioArguments = unsafe { decode_with_size_ptr(pointer) };

        if let Some(app) = &mut self.game {
            let sound = app.render_audio(&render_audio_input);
            return move_ownership_to_host(sound);
        }

        panic!();
    }
}
