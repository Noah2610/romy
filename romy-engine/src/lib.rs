use romy_core::output::*;
use image::GenericImageView;

/// Decode a .png, returning a Image
pub fn decode_png(data: &[u8]) -> Image {
    let image = image::load_from_memory(data).unwrap();
    let rah = image.to_rgba().into_raw();
    Image::from_data(
        image.dimensions().0 as i32,
        image.dimensions().1 as i32,
        &rah,
    )
}

// Decode a .ogg file, retuning a sound for each channel
pub fn decode_ogg(data: &[u8]) -> Vec<Sound> {
    let cursor = std::io::Cursor::new(data);
    let mut srr = lewton::inside_ogg::OggStreamReader::new(cursor).unwrap();
    let channels = srr.ident_hdr.audio_channels;
    let sample_rate = srr.ident_hdr.audio_sample_rate;

    let mut samples = Vec::with_capacity(channels as usize);
    for _ in 0..channels {
        samples.push(Vec::new());
    }

    while let Some(pck_samples) = srr.read_dec_packet().unwrap() {
        for (channel, element) in pck_samples.iter().enumerate() {
            for element in element {
                let frac = f32::from(*element) / f32::from(std::i16::MAX);
                samples[channel].push(frac);
            }
        }
    }

    let mut sounds = Vec::with_capacity(channels as usize);
    for s in samples {
        sounds.push(Sound::from_data(sample_rate as i32, &s));
    }

    sounds
}
