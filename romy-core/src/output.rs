use serde_derive::{Deserialize, Serialize};
use byteorder::{LittleEndian, ReadBytesExt};

pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

impl Color {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
    pub fn as_rgba(&self) -> u32 {
        let range = 255.0;
        let red = (range * self.red) as u32;
        let green = (range * self.green) as u32;
        let blue = (range * self.blue) as u32;
        let alpha = (range * self.alpha) as u32;
        let mut rgba = alpha << 24;
        rgba |= blue << 16;
        rgba |= green << 8;
        rgba |= red;
        rgba
    }
}

/// An image that can be displayed by the runtime.
///
/// Internally stores data as an array of 32 bit RGBA values.
#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
    width: i32,
    height: i32,
    data: Vec<u32>,
}

impl Image {
    /// Create a blank image
    /// # Arguments
    /// * `width` - the number of horizontal pixels
    /// * `height` - the number of vertical pixels.
    /// * `color` - the initial color of all pixels in the image.
    pub fn new(width: i32, height: i32, color: Color) -> Self {
        let color = color.as_rgba();
        let mut d = Vec::with_capacity((width * height) as usize);
        for _ in 0..width * height {
            d.push(color);
        }

        Self {
            width,
            height,
            data: d,
        }
    }

    /// Create an image from a slice of existing data
    /// # Arguments
    /// * `width` - the number of horizontal pixels
    /// * `height` - the number of vertical pixels
    /// * `data` - slice of existing data.
    pub fn from_data(width: i32, height: i32, data: &[u8]) -> Self {
        let mut d = Vec::with_capacity((width * height) as usize);
        for i in 0..width * height {
            let pixel = (&data[i as usize * 4..i as usize * 4 + 4])
                .read_u32::<LittleEndian>()
                .unwrap();

            d.push(pixel);
        }

        Self {
            width,
            height,
            data: d,
        }
    }

    /// Sets a pixel in the image to a specified color
    /// # Arguments
    /// * `x` - horizontal coordinate
    /// * `y` - vertical coordinate
    /// * `color` - color to set the pixel to
    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
        let width = self.width();
        self.pixels_mut()[(y * width + x) as usize] = color.as_rgba();
    }

    /// Gets the number of horizontal pixels
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Gets the number of vertical pixels
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Gets a reference to the raw pixel buffer
    pub fn pixels(&self) -> &[u32] {
        &self.data
    }

    /// Gets a reference to the raw pixel as u8s
    pub fn pixels8(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.data.len() * 4)
        }
    }

    /// Gets a mutable reference to the raw pixel buffer
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.data
    }

    /// Gets a mutable reference to the raw pixel buffer as u8s
    pub fn pixels8_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut u8, self.data.len() * 4)
        }
    }

    /// Draws an image into a section of this one, will ignore fully transparent pixels, but does
    /// not blend semi-transparent ones.
    /// # Arguments
    /// * `source' - Image to take data from
    /// * `x` - horizontal coordinate to draw at in this image
    /// * `y` - vertical coordinate to draw at in this image
    /// * `width` - horizontal pixel span to draw into on this image
    /// * `height` - vertical pixel span to draw into on this image
    pub fn blit(&mut self, source: &Image, x: i32, y: i32, width: i32, height: i32) {
        let input_width = source.width();
        let input_height = source.height();
        let output_width = self.width();
        let draw_at_x = x;
        let draw_at_y = y;

        let x_ratio = input_width as f32 / width as f32;
        let y_ratio = input_height as f32 / height as f32;
        let pixels = source.pixels();

        for y in 0..height {
            for x in 0..width {
                let sample_x = (x as f32 * x_ratio) as i32;
                let sample_y = (y as f32 * y_ratio) as i32;

                let output = self.pixels_mut();

                let o = ((y + draw_at_y) * output_width + x + draw_at_x) as usize;
                if o >= output.len() {
                    continue;
                }
                let i = (sample_y * input_width + sample_x) as usize;
                if i >= pixels.len() {
                    continue;
                }

                if pixels[i] & 0xFF_00_00_00 != 0xFF_00_00_00 {
                    continue;
                }

                output[o] = pixels[i];
            }
        }
    }
}

/// A sound that can be played by the runtime.
///
/// Internally stores data as an array of 32 bit floating point values that range from -1.0 to 1.0
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sound {
    sample_rate: i32,
    samples: Vec<f32>,
}

impl Sound {
    /// Create a sound from a slice of existing data
    /// # Arguments
    /// * `sample_rate` - the number of samples per second
    /// * `data` - slice of existing data
    pub fn from_data(sample_rate: i32, samples: &[f32]) -> Self {
        Self {
            sample_rate,
            samples: samples.to_vec(),
        }
    }
    
    /// Create a blank/silent sound
    /// # Arguments
    /// * `sample_rate` - the number of samples per second
    /// * `sample_count` - the number of samples
    pub fn with_buffer_size(sample_rate: i32, sample_count: i32) -> Self {
        let mut samples = Vec::with_capacity(sample_count as usize);
        samples.resize(sample_count as usize, 0.0);
        Self::from_data(sample_rate, &samples)
    }
    
    /// Create a sound with the number of samples needed to cover a specific step time 
    /// # Arguments
    /// * `sample_rate` - the number of samples per second
    /// * `steps_per_second` - the number of steps per second
    pub fn with_buffer_sized_to_step(sample_rate: i32, steps_per_second: i32) -> Self {
        let sample_count = sample_rate / steps_per_second;
        Self::with_buffer_size(sample_rate, sample_count)
    }

    /// Sets a value of the sample
    /// # Arguments
    /// * `index' sample index
    /// * `sample` - sample value 
    pub fn set_sample(&mut self, index: i32, sample: f32) {
        self.samples_mut()[index as usize] = sample;
    }

    /// Gets the sample rate of the sound
    pub fn sample_rate(&self) -> i32 {
        self.sample_rate
    }

    /// Gets the number of samples stored in this sound
    pub fn sample_count(&self) -> i32 {
        self.samples.len() as i32
    }

    /// Gets a reference to the raw sample data
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Gets a mutable reference to the raw sample data
    pub fn samples_mut(&mut self) -> &mut [f32] {
        &mut self.samples
    }

    /// Creates a new sound by sampling a section of this one
    /// # Arguments
    /// * `start' sample start index
    /// * `length` - length of the sample
    pub fn sample(&self, start: i32, length: i32) -> Self {
        Self {
            sample_rate: self.sample_rate,
            samples: self.samples[(start as usize)..((start + length) as usize)].to_vec(),
        }
    }
}