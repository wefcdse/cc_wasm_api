// use rand::{seq::SliceRandom, thread_rng};

mod drawing;
mod functions;
mod initing;

use crate::prelude::LuaResult;

use super::{
    misc::{AsIfPixel, ColorId, Direction},
    vec2d::Vec2d,
};
pub use initing::InitMethod;

/// a monitor but stores the pixel localy,
/// and can send only changed pixels
///
/// x, y starts with 1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalMonitor {
    pub(crate) data: Vec2d<AsIfPixel>,
    pub(crate) last_sync: Vec2d<AsIfPixel>,
    // pub(crate) side: Side,
    pub(crate) name: String,
    // pub(crate) is_remote: bool,
    // pub(crate) remote_name: Option<String>,
}

// creating
impl LocalMonitor {
    fn name(&self) -> &str {
        &self.name
    }
    pub const fn xy_rate() -> f32 {
        61. / 40.
    }
    pub const fn new_empty() -> Self {
        Self {
            data: Vec2d::new_empty(),
            last_sync: Vec2d::new_empty(),

            // side: Side::Top,
            name: String::new(),
        }
    }
    fn new(x: usize, y: usize, pixel: AsIfPixel, init_method: InitMethod) -> Self {
        Self {
            data: Vec2d::new_filled_copy(x, y, pixel),
            last_sync: Vec2d::new_filled_copy(
                x,
                y,
                AsIfPixel::colored_whitespace(ColorId::from_number_overflow(
                    (pixel.background_color.to_number() + 1).into(),
                )),
            ),
            // side,
            name: LocalMonitor::gen_name(init_method),
        }
    }
    fn resize(&mut self, x: usize, y: usize, pixel: AsIfPixel) {
        self.data = Vec2d::new_filled_copy(x, y, pixel);
        self.last_sync = Vec2d::new_filled_copy(
            x,
            y,
            AsIfPixel::colored_whitespace(ColorId::from_number_overflow(
                (pixel.background_color.to_number() + 1).into(),
            )),
        );
    }
    pub fn size(&self) -> (usize, usize) {
        self.data.size()
    }
    pub fn x(&self) -> usize {
        self.data.x()
    }
    pub fn y(&self) -> usize {
        self.data.y()
    }
}

// useing
impl LocalMonitor {
    /// x, y starts with 1
    pub fn get(&self, x: usize, y: usize) -> Option<AsIfPixel> {
        if x > self.x() || y > self.y() {
            None
        } else {
            let x = x - 1;
            let y = y - 1;
            Some(self.data[(x, y)])
        }
    }
    /// x, y starts with 1
    pub fn write(&mut self, x: usize, y: usize, pixel: AsIfPixel) {
        if x > self.x() || y > self.y() || x == 0 || y == 0 {
            return;
        }
        let x = x - 1;
        let y = y - 1;
        let p0 = self.data[(x, y)];
        if p0 != pixel {
            self.data[(x, y)] = pixel;
        }
    }

    pub fn clear_local(&mut self, color: ColorId) {
        // for x in 1..=self.x() {
        //     for y in 1..=self.y() {
        //         let pixel = AsIfPixel::colored_whitespace(color);
        //         self.write(x, y, pixel);
        //     }
        // }

        self.data.iter_mut().for_each(|(_, pix)| {
            *pix = AsIfPixel::colored_whitespace(color);
        });
    }

    /// write a [str], ignore non-ASCII chars
    pub fn write_str(
        &mut self,
        x: usize,
        y: usize,
        direction: Direction,
        text: &str,
        background_color: ColorId,
        text_color: ColorId,
    ) {
        let (dx, dy) = direction.to_dxdy();
        let (size_x, size_y) = self.size();
        let (size_x, size_y) = (size_x as isize, size_y as isize);

        let mut now_x = x as isize;
        let mut now_y = y as isize;
        #[allow(unused)]
        let (x, y) = ((), ());

        for c in text.chars() {
            let pixel = if let Some(p) = AsIfPixel::new(c, background_color, text_color) {
                p
            } else {
                continue;
            };
            self.write(now_x as usize, now_y as usize, pixel);
            now_x += dx;
            now_y += dy;
            if now_x <= 0 || now_x > size_x || now_y <= 0 || now_y > size_y {
                return;
            }
        }
    }
    pub async fn set_plattle(&self) -> LuaResult<()> {
        Ok(())
    }
}
