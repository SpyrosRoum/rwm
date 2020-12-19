use x11rb::protocol::xproto::ModMask;

use crate::layouts::LayoutType;

#[derive(Debug)]
pub struct Config {
    pub(crate) border_width: u32,
    /// ARGB format
    pub(crate) focused_border_color: u32,
    pub(crate) normal_border_color: u32,
    pub(crate) mod_key: ModMask,
    /// First one is the default
    pub(crate) layouts: Vec<LayoutType>,
    /// If the focus will follow the cursor or not
    pub(crate) follow_cursor: bool,
}

impl Default for Config {
    fn default() -> Self {
        let blue_bytes = [
            255_u8, // Alpha
            000_u8,   // Red
            000_u8,   // Green
            255_u8, // Blue
        ];
        let blue = u32::from_be_bytes(blue_bytes);

        let gray_bytes = [
            255_u8, // Alpha
            211_u8, // Red
            211_u8, // Green
            211_u8, // Blue
        ];
        let gray = u32::from_be_bytes(gray_bytes);

        Self {
            border_width: 4, // pixels
            focused_border_color: blue,
            normal_border_color: gray,
            mod_key: ModMask::M1, // left alt
            layouts: vec![
                LayoutType::MonadTall,
                LayoutType::Grid,
                LayoutType::Floating,
            ],
            follow_cursor: true
        }
    }
}
