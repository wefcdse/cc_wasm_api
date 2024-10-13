// use rand::{seq::SliceRandom, thread_rng};

use crate::{
    eval::{eval, exec},
    prelude::LuaResult,
    utils::Number,
};

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
    last_sync: Vec2d<AsIfPixel>,
    side: Side,
}

// creating
impl LocalMonitor {
    pub const fn new_empty(side: Side) -> Self {
        Self {
            data: Vec2d::new_empty(),
            last_sync: Vec2d::new_empty(),

            side,
        }
    }
    fn new(x: usize, y: usize, pixel: AsIfPixel, side: Side) -> Self {
        Self {
            data: Vec2d::new_filled_copy(x, y, pixel),
            last_sync: Vec2d::new_filled_copy(
                x,
                y,
                AsIfPixel::colored_whitespace(ColorId::from_number_overflow(
                    (pixel.background_color.to_number() + 1).into(),
                )),
            ),
            side,
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
}

// const BATCH: usize = 20000;
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

        let mut to_write = Vec::new();

        for ((x, y), pix) in self.data.iter() {
            if *pix != self.last_sync[(x, y)] {
                to_write.push((x, y, *pix));
                count += 1;
            }
        }

        to_write.sort_by(|a, b| {
            (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color))
        });

        if to_write.is_empty() {
            return Ok(0);
        }

        let mut write_script = functions::set_color(to_write[0].2, self.side);
        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        for (x, y, pix) in to_write {
            let color = (pix.text_color, pix.background_color);
            if color != last_color {
                write_script += &functions::set_color(pix, self.side);
            }
            write_script += &functions::write_txt(x, y, pix, self.side);
            last_color = color;
        }

        exec(&write_script).await?;
        // if !write_script.is_empty() {
        //     exec(&format!("print({})", write_script.len())).await?;
        // }
        self.last_sync = self.data.clone();
        Ok(count)
    }
    pub async fn sync_limited(&mut self, limit: usize) -> LuaResult<usize> {
        let mut count = 0;

        let mut to_write = Vec::new();

        for ((x, y), pix) in self.data.iter() {
            if *pix != self.last_sync[(x, y)] {
                to_write.push((x, y, *pix));
                count += 1;
            }
        }
        // to_write.shuffle(&mut thread_rng());

        to_write.sort_by(|a, b| {
            (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color))
        });

        if to_write.is_empty() {
            return Ok(0);
        }

        let mut write_script = functions::set_color(to_write[0].2, self.side);
        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        for (idx, (x, y, pix)) in to_write.into_iter().enumerate() {
            if idx >= limit {
                break;
            }
            let color = (pix.text_color, pix.background_color);
            if color != last_color {
                write_script += &functions::set_color(pix, self.side);
            }
            write_script += &functions::write_txt(x, y, pix, self.side);
            last_color = color;
            self.last_sync[(x, y)] = pix;
        }

        exec(&write_script).await?;
        // if !write_script.is_empty() {
        //     exec(&format!("print({})", write_script.len())).await?;
        // }
        Ok(count)
    }

    pub async fn sync_limited_rate(&mut self, limit: f32) -> LuaResult<usize> {
        let mut count = 0;

        let mut to_write = Vec::new();

        for ((x, y), pix) in self.data.iter() {
            if *pix != self.last_sync[(x, y)] {
                to_write.push((x, y, *pix));
                count += 1;
            }
        }
        // to_write.shuffle(&mut thread_rng());

        to_write.sort_by(|a, b| {
            (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color))
        });

        if to_write.is_empty() {
            return Ok(0);
        }
        let limit = (limit * to_write.len() as f32).ceil() as usize;
        let mut write_script = functions::set_color(to_write[0].2, self.side);
        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        for (idx, (x, y, pix)) in to_write.into_iter().enumerate() {
            if idx >= limit {
                break;
            }
            let color = (pix.text_color, pix.background_color);
            if color != last_color {
                write_script += &functions::set_color(pix, self.side);
            }
            write_script += &functions::write_txt(x, y, pix, self.side);
            last_color = color;
            self.last_sync[(x, y)] = pix;
        }

        exec(&write_script).await?;
        // if !write_script.is_empty() {
        //     exec(&format!("print({})", write_script.len())).await?;
        // }
        Ok(count)
    }

    async fn _sync_all(&mut self) -> LuaResult<()> {
        let mut write_script = String::new();
        for ((x, y), pixel) in self.data.iter() {
            write_script += &functions::_write_pix(x, y, *pixel, self.side);
        }
        if !write_script.is_empty() {
            exec(&format!("print({})", write_script.len())).await?;
        }
        exec(&write_script).await?;
        self.last_sync = self.data.clone();
        Ok(())
    }

    pub async fn clear(&mut self, color: ColorId) -> LuaResult<()> {
        functions::clear(color, self.side).await?;
        self.data.iter_mut().for_each(|(_, pix)| {
            *pix = AsIfPixel::colored_whitespace(color);
        });
        self.last_sync = self.data.clone();
        Ok(())
    }
}
