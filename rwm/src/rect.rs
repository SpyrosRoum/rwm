#[derive(Debug)]
pub(crate) struct Rect {
    pub(crate) x: i16,
    pub(crate) y: i16,
    pub(crate) width: u16,
    pub(crate) height: u16,
}

impl Rect {
    pub(crate) fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(crate) fn contains_point(&self, x: i16, y: i16) -> bool {
        x >= self.x
            && x as i32 <= self.x as i32 + self.width as i32
            && y >= self.y
            && y as i32 <= self.y as i32 + self.height as i32
    }
}

impl From<(i16, i16, u16, u16)> for Rect {
    fn from(r: (i16, i16, u16, u16)) -> Self {
        Self {
            x: r.0,
            y: r.1,
            width: r.2,
            height: r.3,
        }
    }
}
