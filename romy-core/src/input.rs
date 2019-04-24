use super::*;
use serde_derive::{Deserialize, Serialize};

/// Input device types, will resolve to a InputDevice
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum InputDeviceType {
    /// Nintendo Entertainment System style controller
    Nes,
    /// A modern controller, similar to a XBox 360 controller
    Controller,
    /// A computer keyboard
    Keyboard,
}

/// Trait for converting from one input type to another
pub trait InputConvert {
    /// Returns how closely this device matches the type of another, for example a standard
    /// controller is closely to a NES style controller than a keyboard is. Lower values are closer
    /// fits. None = cant be converted at all.
    fn affinity(&self, device_type: InputDeviceType) -> Option<i32>;

    //Convert this device into another device, None = can't be converted.
    fn convert(&self, device_type: InputDeviceType) -> Option<InputDevice>;
}

/// Trait for combining two inputs together
pub trait InputCombine {
    /// Combine this input with another one, usually this means any pressed buttons from either
    /// device will be down in the new one
    fn combine(&self, with: &Self) -> Self;
}

/// Enumeration over all input types
#[derive(Serialize, Deserialize, Clone)]
pub enum InputDevice {
    Nes(Nes),
    Controller(Controller),
    Keyboard(Keyboard),
}

impl InputCombine for InputDevice {
    fn combine(&self, with: &Self) -> Self {
        match self {
            InputDevice::Nes(nes) => {
                let conversion = with.convert(InputDeviceType::Nes);
                if let Some(conversion) = conversion {
                    if let InputDevice::Nes(with) = conversion {
                        return InputDevice::Nes(nes.combine(&with));
                    }
                }
            }
            InputDevice::Controller(standard_controller) => {
                let conversion = with.convert(InputDeviceType::Controller);
                if let Some(conversion) = conversion {
                    if let InputDevice::Controller(with) = conversion {
                        return InputDevice::Controller(standard_controller.combine(&with));
                    }
                }
            }
            InputDevice::Keyboard(keyboard) => {
                let conversion = with.convert(InputDeviceType::Keyboard);
                if let Some(conversion) = conversion {
                    if let InputDevice::Keyboard(with) = conversion {
                        return InputDevice::Keyboard(keyboard.combine(&with));
                    }
                }
            }
        }

        self.clone()
    }
}

impl InputConvert for InputDevice {
    fn convert(&self, device_type: InputDeviceType) -> Option<InputDevice> {
        match self {
            InputDevice::Nes(nes) => nes.convert(device_type),
            InputDevice::Controller(standard_controller) => {
                standard_controller.convert(device_type)
            }
            InputDevice::Keyboard(keyboard) => keyboard.convert(device_type),
        }
    }
    fn affinity(&self, device_type: InputDeviceType) -> Option<i32> {
        match self {
            InputDevice::Nes(nes) => nes.affinity(device_type),
            InputDevice::Controller(standard_controller) => {
                standard_controller.affinity(device_type)
            }
            InputDevice::Keyboard(keyboard) => keyboard.affinity(device_type),
        }
    }
}

/// Collection of many inputs
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct InputCollection {
    inputs: Vec<InputDevice>,
}

impl InputCollection {
    pub fn new() -> Self {
        Self { inputs: Vec::new() }
    }

    /// Add a new input to the collection
    ///
    /// # Arguments
    /// * `device` - Device to add
    pub fn add_input(&mut self, device: InputDevice) {
        self.inputs.push(device)
    }

    /// Distribute all of the inputs in the collection amongst all of the players mentioned in the
    /// info argument and return a InputArgument suitable for passing to Game::Step()
    ///
    /// # Arguments
    /// * `info` - The game info
    pub fn get_input_arguments(&self, info: &Info) -> InputArguments {
        let devices: Vec<InputDeviceType> = info
            .players
            .iter()
            .map(|player| player.input.clone())
            .collect();

        let (dist, mut remaining) = self.split(&devices);
        let mut result: Vec<Option<PlayerInputArguments>> = dist
            .iter()
            .map(|input| match input {
                Some(input) => Some(PlayerInputArguments {
                    input: input.clone(),
                }),
                None => None,
            })
            .collect();

        //TODO: HORRID LOOP TO COMBINE ALL POSSIBLE INPUTS:
        loop {
            let (new_dist, new_remaining) = remaining.split(&devices);
            if new_remaining.inputs.len() == remaining.inputs.len() {
                break;
            }

            for (result_index, result_player) in result.iter_mut().enumerate() {
                if let Some(player) = result_player {
                    if let Some(device) = &new_dist[result_index] {
                        player.input = player.input.combine(device);
                    }
                }
            }

            remaining = new_remaining;
        }

        InputArguments::new(result)
    }

    /// Splits this collection up into into separate inputs
    ///
    /// # Arguments
    /// * `into` - a slice of inputs type to split into.
    /// Returns a tuple with the split inputs and a collection of remaining ones.
    fn split(&self, into: &[InputDeviceType]) -> (Vec<Option<InputDevice>>, InputCollection) {
        let mut remaining = self.inputs.clone();
        let mut found = Vec::new();

        //TODO: PREFER DEVICES THAT ARE THE RIGHT TYPE:

        for input_type in into {
            let mut found_index = None;
            let mut found_affinity = None;
            let mut found_for = None;
            for (index, input) in remaining.iter().enumerate() {
                let affinity = input.affinity(input_type.clone());
                match affinity {
                    Some(affinity) => {
                        if let Some(fa) = found_affinity {
                            if affinity >= fa {
                                continue;
                            }
                        }

                        let found_new = input.convert(input_type.clone());
                        if let Some(found_new) = found_new {
                            found_affinity = Some(affinity);
                            found_index = Some(index);
                            found_for = Some(found_new);
                        }
                    }
                    None => continue,
                }
            }

            found.push(found_for);

            if let Some(index) = found_index {
                remaining.remove(index);
            }
        }

        (found, InputCollection { inputs: remaining })
    }
}

impl InputConvert for InputCollection {
    fn convert(&self, device_type: InputDeviceType) -> Option<InputDevice> {
        let mut successfully_converted = Vec::new();
        for input in &self.inputs {
            if let Some(input) = input.convert(device_type.clone()) {
                successfully_converted.push(input);
            }
        }

        if successfully_converted.is_empty() {
            None
        } else {
            let mut result = successfully_converted.pop().unwrap();
            while let Some(converted) = successfully_converted.pop() {
                result = result.combine(&converted);
            }

            Some(result)
        }
    }
    fn affinity(&self, device_type: InputDeviceType) -> Option<i32> {
        let mut affinity = None;
        for input in &self.inputs {
            match affinity {
                Some(current) => {
                    if let Some(test) = input.affinity(device_type.clone()) {
                        if test < current {
                            affinity = Some(test);
                        }
                    }
                }
                None => affinity = input.affinity(device_type.clone()),
            }
        }
        affinity
    }
}

impl InputCombine for InputCollection {
    fn combine(&self, with: &Self) -> Self {
        let mut inputs = self.inputs.clone();
        inputs.extend(with.inputs.clone());
        Self { inputs }
    }
}

/// An input type similar to a Nintendo Entertainment System controller, has a dpad and 2 primary
/// buttons. Also has start + select.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Nes {
    a: bool,
    b: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    start: bool,
    select: bool,
}

impl Nes {
    /// Is the a button down
    pub fn a(&self) -> bool {
        self.a
    }

    /// Is the b button down
    pub fn b(&self) -> bool {
        self.b
    }

    /// Is the up button down
    pub fn up(&self) -> bool {
        self.up
    }

    /// Is the down button down
    pub fn down(&self) -> bool {
        self.down
    }

    /// Is the left button down
    pub fn left(&self) -> bool {
        self.left
    }

    /// Is the right button down
    pub fn right(&self) -> bool {
        self.right
    }

    /// Is the start button down
    pub fn start(&self) -> bool {
        self.start
    }

    /// Is the select button down
    pub fn select(&self) -> bool {
        self.select
    }

    /// Sets the state of the a button
    pub fn set_a(&mut self, value: bool) {
        self.a = value;
    }

    /// Sets the state of the b button
    pub fn set_b(&mut self, value: bool) {
        self.b = value;
    }

    /// Sets the state of the up button
    pub fn set_up(&mut self, value: bool) {
        self.up = value;
    }

    /// Sets the state of the down button
    pub fn set_down(&mut self, value: bool) {
        self.down = value;
    }

    /// Sets the state of the left button
    pub fn set_left(&mut self, value: bool) {
        self.left = value;
    }

    /// Sets the state of the right button
    pub fn set_right(&mut self, value: bool) {
        self.right = value;
    }

    /// Sets the state of the start button
    pub fn set_start(&mut self, value: bool) {
        self.start = value;
    }

    /// Sets the state of the select button
    pub fn set_select(&mut self, value: bool) {
        self.select = value;
    }
}

impl InputConvert for Nes {
    fn convert(&self, device_type: InputDeviceType) -> Option<InputDevice> {
        match device_type {
            InputDeviceType::Nes => Some(InputDevice::Nes(self.clone())),
            _ => None,
        }
    }
    fn affinity(&self, device_type: InputDeviceType) -> Option<i32> {
        match device_type {
            InputDeviceType::Nes => Some(0),
            _ => None,
        }
    }
}

impl InputCombine for Nes {
    fn combine(&self, with: &Self) -> Self {
        Self {
            a: self.a || with.a,
            b: self.b || with.b,
            up: self.up || with.up,
            down: self.down || with.down,
            left: self.left || with.left,
            right: self.right || with.right,
            start: self.start || with.start,
            select: self.select || with.select,
        }
    }
}

/// A standard controller, similar to one used for a XBox 360
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Controller {
    a: bool,
    b: bool,
    x: bool,
    y: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    start: bool,
    select: bool,
    guide: bool,
    left_shoulder: bool,
    right_shoulder: bool,
    left_stick: bool,
    right_stick: bool,
    left_stick_x: f32,
    left_stick_y: f32,
    right_stick_x: f32,
    right_stick_y: f32,
    left_trigger: f32,
    right_trigger: f32,
}

/// Structure for initializing a controller
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ControllerInit {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub start: bool,
    pub select: bool,
    pub guide: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    pub left_stick: bool,
    pub right_stick: bool,
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub left_trigger: f32,
    pub right_trigger: f32,
}

impl Controller {
    pub fn new(init: ControllerInit) -> Self {
        Controller {
            a: init.a,
            b: init.b,
            x: init.x,
            y: init.y,
            up: init.up,
            down: init.down,
            left: init.left,
            right: init.right,
            start: init.start,
            select: init.select,
            guide: init.guide,
            left_shoulder: init.left_shoulder,
            right_shoulder: init.right_shoulder,
            left_stick: init.left_stick,
            right_stick: init.right_stick,
            left_stick_x: init.left_stick_x,
            left_stick_y: init.left_stick_y,
            right_stick_x: init.right_stick_x,
            right_stick_y: init.right_stick_y,
            left_trigger: init.left_trigger,
            right_trigger: init.right_trigger,
        }
    }

    /// Is the a button down
    pub fn a(&self) -> bool {
        self.a
    }

    /// Is the b button down
    pub fn b(&self) -> bool {
        self.b
    }

    /// Is the x button down
    pub fn x(&self) -> bool {
        self.x
    }

    /// Is the y button down
    pub fn y(&self) -> bool {
        self.y
    }

    /// Is the up button down
    pub fn up(&self) -> bool {
        self.up
    }

    /// Is the down button down
    pub fn down(&self) -> bool {
        self.down
    }

    /// Is the left button down
    pub fn left(&self) -> bool {
        self.left
    }

    /// Is the right button down
    pub fn right(&self) -> bool {
        self.right
    }

    /// Is the start button down
    pub fn start(&self) -> bool {
        self.start
    }

    /// Is the select button down
    pub fn select(&self) -> bool {
        self.select
    }

    /// Is the guide button down
    pub fn guide(&self) -> bool {
        self.guide
    }

    /// Is the left shoulder button down
    pub fn left_shoulder(&self) -> bool {
        self.left_shoulder
    }

    /// Is the right shoulder button down
    pub fn right_shoulder(&self) -> bool {
        self.right_shoulder
    }

    /// Is the left stick button down
    pub fn left_stick(&self) -> bool {
        self.left_stick
    }

    /// Is the right stick button down
    pub fn right_stick(&self) -> bool {
        self.right_stick
    }

    /// Left stick position x [-1 - +1] -1 left +1 right
    pub fn left_stick_x(&self) -> f32 {
        self.left_stick_x
    }

    /// Left stick position y [-1 - +1] -1 up +1 down
    pub fn left_stick_y(&self) -> f32 {
        self.left_stick_y
    }

    /// Right stick position x [-1 - +1] -1 left +1 right
    pub fn right_stick_x(&self) -> f32 {
        self.right_stick_x
    }

    /// Right stick position y [-1 - +1] -1 up +1 down
    pub fn right_stick_y(&self) -> f32 {
        self.right_stick_y
    }

    /// Left trigger position [0 - +1] +1 fully down
    pub fn left_trigger(&self) -> f32 {
        self.left_trigger
    }

    /// Right trigger position [0 - +1] +1 fully down
    pub fn right_trigger(&self) -> f32 {
        self.right_trigger
    }

    /// Sets the state of the a button
    pub fn set_a(&mut self, value: bool) {
        self.a = value;
    }

    /// Sets the state of the b button
    pub fn set_b(&mut self, value: bool) {
        self.b = value;
    }

    /// Sets the state of the x button
    pub fn set_x(&mut self, value: bool) {
        self.x = value;
    }

    /// Sets the state of the y button
    pub fn set_y(&mut self, value: bool) {
        self.y = value;
    }

    /// Sets the state of the up button
    pub fn set_up(&mut self, value: bool) {
        self.up = value;
    }

    /// Sets the state of the down button
    pub fn set_down(&mut self, value: bool) {
        self.down = value;
    }

    /// Sets the state of the left button
    pub fn set_left(&mut self, value: bool) {
        self.left = value;
    }

    /// Sets the state of the right button
    pub fn set_right(&mut self, value: bool) {
        self.right = value;
    }

    /// Sets the state of the start button
    pub fn set_start(&mut self, value: bool) {
        self.start = value;
    }

    /// Sets the state of the select button
    pub fn set_select(&mut self, value: bool) {
        self.select = value;
    }

    /// Sets the state of the guide button
    pub fn set_guide(&mut self, value: bool) {
        self.guide = value;
    }

    /// Sets the state of the left shoulder button
    pub fn set_left_shoulder(&mut self, value: bool) {
        self.left_shoulder = value;
    }

    /// Sets the state of the right shoulder button
    pub fn set_right_shoulder(&mut self, value: bool) {
        self.right_shoulder = value;
    }

    /// Sets the state of the left stick button
    pub fn set_left_stick(&mut self, value: bool) {
        self.left_stick = value;
    }

    /// Sets the state of the right stick button
    pub fn set_right_stick(&mut self, value: bool) {
        self.right_stick = value;
    }

    /// Sets the state of the left stick x axis
    pub fn set_left_stick_x(&mut self, value: f32) {
        self.left_stick_x = value;
    }
    
    /// Sets the state of the left stick y axis
    pub fn set_left_stick_y(&mut self, value: f32) {
        self.left_stick_y = value;
    }
    
    /// Sets the state of the right stick x axis
    pub fn set_right_stick_x(&mut self, value: f32) {
        self.right_stick_x = value;
    }
    
    /// Sets the state of the right stick y axis
    pub fn set_right_stick_y(&mut self, value: f32) {
        self.right_stick_y = value;
    }

    /// Sets the state of the left trigger
    pub fn set_left_trigger(&mut self, value: f32) {
        self.left_trigger = value;
    }
    
    /// Sets the state of the right trigger
    pub fn set_right_trigger(&mut self, value: f32) {
        self.right_trigger = value;
    }
}

impl InputConvert for Controller {
    fn convert(&self, device_type: InputDeviceType) -> Option<InputDevice> {
        match device_type {
            InputDeviceType::Nes => Some(InputDevice::Nes(self.to_nes())),
            InputDeviceType::Controller => Some(InputDevice::Controller(self.clone())),
            _ => None,
        }
    }
    fn affinity(&self, device_type: InputDeviceType) -> Option<i32> {
        match device_type {
            InputDeviceType::Nes => Some(1),
            InputDeviceType::Controller => Some(0),
            _ => None,
        }
    }
}

impl InputCombine for Controller {
    fn combine(&self, with: &Self) -> Self {
        Self {
            a: self.a || with.a,
            b: self.b || with.b,
            x: self.x || with.x,
            y: self.y || with.y,
            up: self.up || with.up,
            down: self.down || with.down,
            left: self.left || with.left,
            right: self.right || with.right,
            start: self.start || with.start,
            select: self.select || with.select,
            guide: self.guide || with.guide,
            left_shoulder: self.left_shoulder || with.left_shoulder,
            right_shoulder: self.right_shoulder || with.right_shoulder,
            left_stick: self.left_stick || with.left_stick,
            right_stick: self.right_stick || with.right_stick,
            left_stick_x: self.left_stick_x.max(with.left_stick_x),
            left_stick_y: self.left_stick_y.max(with.left_stick_y),
            right_stick_x: self.right_stick_x.max(with.right_stick_x),
            right_stick_y: self.right_stick_y.max(with.right_stick_y),
            left_trigger: self.left_trigger.max(with.left_trigger),
            right_trigger: self.right_trigger.max(with.right_trigger),
        }
    }
}

impl Controller {
    fn to_nes(&self) -> Nes {
        let stick_sensitivity = 0.5;

        Nes {
            a: self.a,
            b: self.b,
            up: self.up || self.left_stick_y < -stick_sensitivity,
            down: self.down || self.left_stick_y >= stick_sensitivity,
            left: self.left || self.left_stick_x < -stick_sensitivity,
            right: self.right || self.left_stick_x >= stick_sensitivity,
            start: self.start,
            select: self.select,
        }
    }
}

/// A input for a computer keyboard
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Keyboard {
    pressed: Vec<Key>,
}

impl Keyboard {
    /// Add a key down state
    ///
    /// # Arguments
    /// * `key` - the key that is down
    pub fn key_down(&mut self, key: Key) {
        self.key_up(key.scan_code);
        self.pressed.push(key);
    }

    /// Return a key to the up state
    ///
    /// # Arguments
    /// * `scan_code` - the scan code for the key that is now up
    pub fn key_up(&mut self, scan_code: KeyCode) {
        self.pressed.retain(|key| key.scan_code != scan_code);
    }

    /// Get the pressed state of a key via its scan code, this is not effected by the set locale
    ///
    /// # Arguments
    /// * `scan_code` - The scan code to look up
    pub fn is_down_scan(&self, scan_code: KeyCode) -> bool {
        for key in &self.pressed {
            if key.scan_code == scan_code {
                return true;
            }
        }

        false
    }

    /// Get the pressed state of a key via its key code, this is effected by the set locale
    ///
    /// # Arguments
    /// * `key_code` - The key code to look up
    pub fn is_down_key(&self, key_code: KeyCode) -> bool {
        for key in &self.pressed {
            if key.key_code == key_code {
                return true;
            }
        }

        false
    }

    fn to_nes(&self) -> Nes {
        Nes {
            a: self.is_down_scan(KeyCode::K)
                || self.is_down_scan(KeyCode::X)
                || self.is_down_scan(KeyCode::J),
            b: self.is_down_scan(KeyCode::J)
                || self.is_down_scan(KeyCode::Z)
                || self.is_down_scan(KeyCode::N),
            up: self.is_down_scan(KeyCode::W)
                || self.is_down_scan(KeyCode::Up)
                || self.is_down_scan(KeyCode::F),
            down: self.is_down_scan(KeyCode::S) || self.is_down_scan(KeyCode::Down),
            left: self.is_down_scan(KeyCode::A)
                || self.is_down_scan(KeyCode::Left)
                || self.is_down_scan(KeyCode::R),
            right: self.is_down_scan(KeyCode::D)
                || self.is_down_scan(KeyCode::Right)
                || self.is_down_scan(KeyCode::T),
            start: self.is_down_scan(KeyCode::Enter),
            select: self.is_down_scan(KeyCode::Tab),
        }
    }
}

impl InputConvert for Keyboard {
    fn convert(&self, device_type: InputDeviceType) -> Option<InputDevice> {
        match device_type {
            InputDeviceType::Nes => Some(InputDevice::Nes(self.to_nes())),
            InputDeviceType::Keyboard => Some(InputDevice::Keyboard(self.clone())),
            _ => None,
        }
    }
    fn affinity(&self, device_type: InputDeviceType) -> Option<i32> {
        match device_type {
            InputDeviceType::Nes => Some(2),
            InputDeviceType::Keyboard => Some(0),
            _ => None,
        }
    }
}

impl InputCombine for Keyboard {
    fn combine(&self, with: &Self) -> Self {
        let pressed = self.pressed.clone();

        let mut result = Self { pressed };

        for key in &with.pressed {
            result.key_down(key.clone());
        }

        result
    }
}

/// A key that can be pressed by a Keyboard
#[derive(Serialize, Deserialize, Clone)]
pub struct Key {
    scan_code: KeyCode,
    key_code: KeyCode,
}

impl Key {
    /// Create new key from a scan and key code
    ///
    /// # Arguments
    /// * `scan_code` - The scan code of the key, not effected by the locale
    /// * `key_code` - The key code of the key, effected by the locale
    pub fn new(scan_code: KeyCode, key_code: KeyCode) -> Self {
        Self {
            scan_code,
            key_code,
        }
    }
}

/// Key/scan codes
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum KeyCode {
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    _0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Tab,
    LeftBracket,
    RightBracket,
    Slash,
    Backslash,
    Comma,
    Period,
    Semicolon,
    Quote,
}
