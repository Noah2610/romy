# WebAssembly API - Version 1
Wasm games supported by Romy have have no imports and the following exported functions:

* `func $romy_api_version (result i32)`

Returns the version of this API being used by the game. Will be 1 if following this spec.

* `func $allocate (param i32) (result i32)`

Allocates some space inside the modules default memory and returns a pointer to this memory. The size of the requested memory in bytes is given as the parameter.

* `func $deallocate (param i32)`

Frees memory that was allocated with `$allocate`, the pointer returned from `$allocate` is passed as the parameter.

* `func $init (result i32)`

Initializes the game and returns a pointer into the modules default memory that points to an encoded `Info` structure. The runtime is responsible for calling deallocate on the returned data when it is done using it.

```
Info {
    // The name of the game
    name: String,
    // The requested step interval of the game in nanoseconds.
    step_interval: u32, 
    // Vector of player information
    players: Vec<Player>,
}
Player {
    // Requested input device for player, this should be honored when constructing StepArguments
    // for step()
    input: InputDeviceType, 
}
enum InputDeviceType {
    Nes,
    Controller,
    Keyboard,
}
```

* `func $step (param i32)`

Simulates the game forward one step, the param is a pointer to an encoded `StepArguments` structure.  The runtime is responsible for calling deallocate on the parameter data, this can be done safely after the function returns.

```
StepArguments {
    input: InputArguments,
}
InputArguments {
    players: Vec<Option<PlayerInputArguments>>,
}
PlayerInputArguments {
    input: InputDevice,
}
enum InputDevice {
    Nes(Nes),
    Controller(Controller),
    Keyboard(Keyboard),
}
Nes {
    a: bool,
    b: bool,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    start: bool,
    select: bool,
}
Controller {
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
Keyboard {
    pressed: Vec<Key>,
}
Key {
    scan_code: KeyCode,
    key_code: KeyCode,
}
enum KeyCode {
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
```

* `func $draw (param i32) (result i32)`

Creates a rendering of the game, the param is a pointer to an encoded `DrawArguments` structure and the return value is a pointer to an encoded `Image` structure. The runtime is responsible for calling deallocate on the parameter and return data. The memory passed as the parameter can be freed when the call returns.

```
DrawArguments {
    // The width of the area being drawn to, the Image returned from draw() does not have to be 
    // this same width.
    width: i32,
    // The height of the area being drawn to, the Image returned from draw() does not have to be 
    // this same height.
    height: i32,
    // The amount of time that has passed since the last call to step(), this should be in the 
    // range of 0 - 1 where 0 = no time and 1 = the amount of time given by init().step_interval
    step_offset: f32,
}

Image {
    width: i32,
    height: i32,
    // A vector of width*height number of u32s representing pixels, each pixel is a 32bit 
    // R8G8B8A8 color.
    data: Vec<u32>,
}
```

* `func $render_audio (param i32) (result i32)`

Creates a chunk of sound spanning one step, the param is a pointer to an encoded `RenderAudioArguments` structure and the return value is a pointer to an encoded `Sound` structure. The runtime is responsible for calling deallocate on the parameter and return data. The memory passed as the parameter can be freed when the call returns.

```
RenderAudioArguments {
    // Empty at the moment
}

Sound {
    // The number of samples per second.
    sample_rate: i32,
    // A vector of samples, theres should be enough to cover a the span of time between step() 
    // calls. The samples range from -1 to 1 in amplitude.
    samples: Vec<f32>,
}
```

## Additional Data Types and Encoding

All data passed to and returned from the Wasm instance encode values that exist in the Wasm spec in the same way they are usually stored in its memory, additionally Romy adds some types:

`u64` is a 8 byte little-endian unsigned integer.

`Vec<T>` is a vector of values of type T, it will start with a u64 specifying the number of items in the vector, the bytes following will be the items contained within.

`enum { X, Y, Z }` is an encoding of either a X, Y or Z. The variant index is encoded first as a u32. And then the actual value follows.

`Option<T>` An optional value, a u32 will be at the front of the data that is 0 if there is no value and 1 if there is, if this is 1 the data for T will come next.

`String` will start with a u64 identifying the number of bytes used by the string, the bytes that follow will be the string encoded as UTF-8.

All encoded parameters and return values start with a u64 stating the number of bytes used to hold the parameter or return value. This number does not include the 8 bytes used to store number itself.
