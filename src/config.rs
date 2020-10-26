#[derive(Debug)]
pub struct Config {
    pub(crate) border_width: u32,
    pub(crate) border_color: u32, // ARGB format
}

impl Default for Config {
    fn default() -> Self {
        let bytes: [u8; 4] = [
            255u8.to_be_bytes()[0], // Alpha
            0u8.to_be_bytes()[0],   // Red
            0u8.to_be_bytes()[0],   // Green
            255u8.to_be_bytes()[0], // Blue
        ];

        let blue = u32::from_be_bytes(bytes);
        Self {
            border_width: 4,
            border_color: blue,
        }
    }
}
