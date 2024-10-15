use crate::{
    addon::misc::{AsIfPixel, ColorId, Side},
    debug::{self, show_str},
    eval::{eval, exec},
    prelude::LuaResult,
    utils::Number,
};

use super::{functions, LocalMonitor};

impl LocalMonitor {
    pub(crate) fn gen_nonsynced(&self) -> Vec<(usize, usize, AsIfPixel)> {
        let mut to_write: Vec<(usize, usize, AsIfPixel)> = Vec::new();

        for ((x, y), pix) in self.data.iter() {
            let last_pix = self.last_sync[(x, y)];
            if !(*pix == last_pix
                || (pix.is_whitespace()
                    && last_pix.is_whitespace()
                    && pix.background_color == last_pix.background_color))
            {
                to_write.push((x, y, *pix));
            }
        }
        to_write
    }
}
// script gen
#[allow(dead_code)]
impl LocalMonitor {
    pub(crate) fn gen_draw_basic(
        &self,
        mut draw: Vec<(usize, usize, AsIfPixel)>,
    ) -> (String, usize) {
        draw.sort_by(|a, b| {
            (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color))
        });
        let to_write = draw;

        let (mut write_script, mut code_line) = self.gen_script_set_color(to_write[0].2);
        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        for (x, y, pix) in to_write.iter().copied() {
            let color = (pix.text_color, pix.background_color);
            if color != last_color {
                let (s, c) = self.gen_script_set_color(pix);
                write_script += &s;
                code_line += c;
            }
            let (s, c) = self.gen_script_write_txt(x, y, pix);
            write_script += &s;
            code_line += c;
            last_color = color;
        }
        (write_script, code_line)
    }
    /// x, y, pix
    pub(crate) fn gen_draw_opt_cursor(
        &self,
        mut draw: Vec<(usize, usize, AsIfPixel)>,
    ) -> (String, usize) {
        draw.sort_by(|a, b| {
            let color_cmp =
                (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color));
            if color_cmp.is_ne() {
                return color_cmp;
            }

            let y_cmp = a.2.cmp(&b.2);
            if y_cmp.is_ne() {
                return y_cmp;
            };
            a.1.cmp(&b.1)
        });

        let to_write = draw;
        let (mut write_script, mut code_line) = (String::new(), 0);

        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        {
            let (s, c) = self.gen_script_set_color(to_write[0].2);
            write_script += &s;
            code_line += c;
        }
        let mut cursor_pos = (to_write[0].0, to_write[0].1);
        {
            let (s, c) = self.gen_script_set_cursor(to_write[0].0, to_write[0].1);
            write_script += &s;
            code_line += c;
        }
        for (x, y, pix) in to_write.iter().copied() {
            let color = (pix.text_color, pix.background_color);
            if color != last_color {
                let (s, c) = self.gen_script_set_color(pix);
                write_script += &s;
                code_line += c;
            }
            last_color = color;

            let pos = (x, y);
            if pos != cursor_pos {
                let (s, c) = self.gen_script_set_cursor(x, y);
                write_script += &s;
                code_line += c;
            }
            cursor_pos = (x + 1, y);

            let (s, c) = self.gen_script_write_char(pix);
            write_script += &s;
            code_line += c;
        }
        (write_script, code_line)
    }
    /// x, y, pix
    pub(crate) fn gen_draw_opt_cursor_long_str(
        &self,
        mut draw: Vec<(usize, usize, AsIfPixel)>,
    ) -> (String, usize) {
        if draw.is_empty() {
            return Default::default();
        }

        draw.sort_by(|a, b| {
            let color_cmp =
                (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color));
            if color_cmp.is_ne() {
                return color_cmp;
            }
            let (xa, ya) = (a.0, a.1);
            let (xb, yb) = (b.0, b.1);
            let y_cmp = ya.cmp(&yb);
            if y_cmp.is_ne() {
                return y_cmp;
            };
            xa.cmp(&xb)
        });

        let to_write = draw;
        let (mut write_script, mut code_line) = (String::new(), 0);

        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        {
            let (s, c) = self.gen_script_set_color(to_write[0].2);
            write_script += &s;
            code_line += c;
        }
        let mut cursor_pos = (to_write[0].0, to_write[0].1);
        {
            let (s, c) = self.gen_script_set_cursor(to_write[0].0, to_write[0].1);
            write_script += &s;
            code_line += c;
        }
        let mut write_str = String::new();
        for (x, y, pix) in to_write.iter().copied() {
            let color = (pix.text_color, pix.background_color);
            let pos = (x, y);

            if color != last_color || pos != cursor_pos {
                let (s, c) = self.gen_script_write_multi_char(&write_str);

                write_script += &s;
                code_line += c;
                write_str.clear();
            }

            if color != last_color {
                let (s, c) = self.gen_script_set_color(pix);
                write_script += &s;
                code_line += c;
            }

            if pos != cursor_pos {
                let (s, c) = self.gen_script_set_cursor(x, y);
                write_script += &s;
                code_line += c;
            }

            write_str.push(pix.text());

            last_color = color;
            cursor_pos = (x + 1, y);
            if y == self.y() {
                cursor_pos = (1, y + 1)
            }
        }
        {
            let (s, c) = self.gen_script_write_multi_char(&write_str);
            write_script += &s;
            code_line += c;
            write_str.clear();
        }
        (write_script, code_line)
    }

    pub(crate) fn gen_draw_opt_auto_clear(
        &self,
        draw: Vec<(usize, usize, AsIfPixel)>,
    ) -> (String, usize) {
        let (default_s, default_c) = self.gen_draw_opt_cursor_long_str(draw);

        let mut color_map = [0; 16];
        self.data.iter().for_each(|(_, p)| {
            if p.is_whitespace() {
                color_map[p.background_color] += 1;
            }
        });

        // show_str("after iter");
        let clear_color = {
            let mut max = 0;
            let mut max_idx = 0;
            for (i, c) in color_map.iter().copied().enumerate() {
                if c > max {
                    max = c;
                    max_idx = i;
                }
            }
            ColorId::from_number_overflow(max_idx as u32)
        };
        // show_str("after color");

        let cleared = {
            let mut to_write: Vec<(usize, usize, AsIfPixel)> = Vec::new();

            for ((x, y), pix) in self.data.iter() {
                if pix.background_color != clear_color || !pix.is_whitespace() {
                    to_write.push((x, y, *pix));
                }
            }
            to_write
        };
        // show_str("after changed");

        let (clear_s, clear_c) = {
            // show_str("a");
            let (mut s, mut c) = self.gen_script_clear(clear_color);
            // show_str("b");
            let d = self.gen_draw_opt_cursor_long_str(cleared);
            // show_str("c");
            s += &d.0;
            c += d.1;
            // show_str("d");
            (s, c)
        };
        // show_str("after script gen");
        if clear_c >= default_c {
            show_str(&format!("{} use default", self.name()));
            (default_s, default_c)
        } else {
            show_str(&format!("{} use clear", self.name()));
            (clear_s, clear_c)
        }
    }
}

// const BATCH: usize = 20000;
impl LocalMonitor {
    pub async fn init(&mut self, side: Side) -> LuaResult<()> {
        self.side = side;
        self.name = format!("monitor{}", self.side.name());
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
        let to_write: Vec<(usize, usize, AsIfPixel)> = self.gen_nonsynced();

        if to_write.is_empty() {
            return Ok(0);
        }

        let changed_pix = to_write.len();
        let (write_script, code_line) = self.gen_draw_opt_auto_clear(to_write);

        exec(&write_script).await?;
        debug::show_str(&format!(
            "monitor {} draw code line: {}, changed pix: {}",
            self.name(),
            code_line,
            changed_pix
        ));

        self.last_sync = self.data.clone();
        Ok(changed_pix)
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
