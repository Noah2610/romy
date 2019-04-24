// Bring in Romy:
use romy::*;

// Initialize Romy; we set up a title for our game, a step rate, the number of players and the type
// of input device to use. InputDeviceType::Nes is modeled after a simple 10 button Nintendo
// controller, the runtime will try its best to map any physical inputs to this. The romy_main
// macro creates the necessary Wasm exports and the main() function if building natively.
const STEPS_PER_SECOND: i32 = 60;
romy_main!(
    // Info about our game:
    Info::new(
        "Simple Romy Demo",
        STEPS_PER_SECOND,
        1,
        InputDeviceType::Nes
    ),
    // Make the struct that implements the Game trait that will be controlled by Romy
    Demo::create()
);

// Game state that we would like to maintain between steps, we are creating a game where you can
// move a cool little square around a sea of darkness. We'll store the position of our hero in the 
// x and y fields and the bounds she can move inside in the width and height fields. The sound
// buffer will hold the sound we have generated for Romy to retrieve and play, We'll generate 
// sounds based on where our hero is.
pub struct Demo {
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    sound: Sound,
}

impl Demo {
    pub fn create() -> Self {
        // Create a sound buffer to hold the sound for our game, we want set the sample rate for
        // that here and also ask for the buffer to be sized to hold one step worth of samples.
        // Romy asks for samples once every step.
        let sound = Sound::with_buffer_sized_to_step(44100, STEPS_PER_SECOND);

        // Our initial state:
        Self {
            width: 128,
            height: 128,
            x: 5,
            y: 5,
            sound,
        }
    }
}

impl Game for Demo {
    // This is called at the rate specified during initialization, 60 times a second in our case.
    // We are free to modify memory/state here:
    fn step(&mut self, arguments: &StepArguments) {
        // Get the Nes style controller for the first player:
        let controller = arguments.input().player(0).and_then(|player| player.nes());

        // If we have a controller, move our hero based on what buttons are pressed, make sure they
        // stay inside the bounds of the game world:
        if let Some(controller) = controller {
            let speed = 1;
            if controller.up() && self.y > 0 {
                self.y -= speed;
            }
            if controller.down() && self.y < self.height - speed {
                self.y += speed;
            }
            if controller.left() && self.x > 0 {
                self.x -= speed;
            }
            if controller.right() && self.x < self.width - speed {
                self.x += speed;
            }
        }

        // Fill up our sound buffer for this step, we are creating a sine wave here with
        // a higher frequency/pitch the further to the right the hero is.
        let samples = self.sound.samples_mut();
        let sample_count = samples.len();
        for (index, sample) in samples.iter_mut().enumerate() {
            let cycle_per_step = (index as f32 / sample_count as f32) * std::f32::consts::PI * 2.0;
            let scaled_by_position = cycle_per_step * (self.x as f32 * 0.25).round();
            *sample = f32::sin(scaled_by_position);
        }
    }

    // This is called every time Romy would like to display a new image, the rate this is called is 
    // not tied to the step rate. It's up to Romy to decide how often it'd like new images, usually
    // this would be at the refresh rate of the display it's showing them on. Memory/state can't
    // be modified here, but arguments.step_offset is a value with a range of 0.0 - 1.0 indicating 
    // how far through to the next step we are, and as such can be used to create smooth animations.
    // We are free to return an image of any size, Romy will sort it out. The current display 
    // resolution is passed in as arguments.width and arguments.height if you would like to use
    // those. State can't be changed in here.
    fn draw(&self, _arguments: &DrawArguments) -> Image {
        // Create a new image:
        let mut display = Image::new(self.width, self.height, Color::new(0.2, 0.2, 0.2, 1.0));

        // Display our hero:
        display.set_pixel(self.x, self.y, Color::new(1.0, 1.0, 1.0, 1.0));
        display
    }

    // This is called when Romy wants some sound to play, it will be called at most once per step,
    // we are expected to supply enough audio for the duration of a step, state can't be changed 
    // in here.
    fn render_audio(&self, _arguments: &RenderAudioArguments) -> Sound {
        self.sound.clone()
    }
}