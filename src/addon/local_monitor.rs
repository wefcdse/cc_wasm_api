use crate::{eval::eval, prelude::LuaResult, utils::Number};

use super::{
    functions,
    misc::{AsIfPixel, ColorId, Direction, Side},
    vec2d::Vec2d,
};

/// a monitor but stores the pixel localy,
/// and can send only changed pixels
///
/// x, y starts with 1
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalMonitor {
    data: Vec2d<AsIfPixel>,
    changed: Vec2d<bool>,
    side: Side,
}

// creating
impl LocalMonitor {
    pub const fn new_empty(side: Side) -> Self {
        Self {
            data: Vec2d::new_empty(),
            changed: Vec2d::new_empty(),

            side,
        }
    }
    fn new(x: usize, y: usize, pixel: AsIfPixel, side: Side) -> Self {
        Self {
            data: Vec2d::new_filled_copy(x, y, pixel),
            changed: Vec2d::new_filled_copy(x, y, true),
            side,
        }
    }
    fn resize(&mut self, x: usize, y: usize, pixel: AsIfPixel) {
        self.data = Vec2d::new_filled_copy(x, y, pixel);
        self.changed = Vec2d::new_filled_copy(x, y, true);
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
            self.changed[(x, y)] = true;
        }
    }

    pub fn clear_with(&mut self, color: ColorId) {
        for x in 1..=self.x() {
            for y in 1..=self.y() {
                let pixel = AsIfPixel::colored_whitespace(color);
                self.write(x, y, pixel);
            }
        }
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
}

impl LocalMonitor {
    pub async fn init(&mut self) -> LuaResult<()> {
        let inited = Self::new_inited(self.side).await?;
        *self = inited;
        Ok(())
    }
    pub async fn new_inited(side: Side) -> LuaResult<Self> {
        functions::init_monitor(side).await?;
        let (x, y): (Number, Number) =
            eval(&format!("return monitor{}.getSize()", side.name())).await?;
        let (x, y) = (x.to_i32() as usize, y.to_i32() as usize);

        let mut new_self = Self::new(x, y, AsIfPixel::default(), side);
        new_self
            .clear(AsIfPixel::default().background_color)
            .await?;

        Ok(new_self)
    }
    /// returns if resized
    pub async fn sync_size(&mut self) -> LuaResult<bool> {
        let (x, y): (Number, Number) =
            eval(&format!("return monitor{}.getSize()", self.side.name())).await?;
        let (x, y) = (x.to_i32() as usize, y.to_i32() as usize);
        if self.size() == (x, y) {
            return Ok(false);
        }
        self.resize(x, y, AsIfPixel::default());
        // let mut new_self = Self::new(x, y, AsIfPixel::default(), side);
        self.clear(AsIfPixel::default().background_color).await?;

        Ok(true)
    }
    pub async fn sync(&mut self) -> LuaResult<usize> {
        let mut count = 0;

        for ((x, y), changed) in self.changed.iter() {
            if *changed {
                let pixel = self.data[(x, y)];

                functions::write_pix(x as i32, y as i32, pixel, self.side).await?;

                count += 1;
            }
        }

        self.changed = Vec2d::new_filled_copy(self.x(), self.y(), false);
        Ok(count)
    }

    pub async fn sync_all(&mut self) -> LuaResult<()> {
        for ((x, y), pixel) in self.data.iter() {
            functions::write_pix(x as i32, y as i32, *pixel, self.side).await?;
        }
        self.changed = Vec2d::new_filled_copy(self.x(), self.y(), false);
        Ok(())
    }

    pub async fn clear(&mut self, color: ColorId) -> LuaResult<()> {
        functions::clear(color, self.side).await?;
        self.data.iter_mut().for_each(|(_, pix)| {
            *pix = AsIfPixel::colored_whitespace(color);
        });
        self.changed = Vec2d::new_filled_copy(self.x(), self.y(), false);
        Ok(())
    }
}
