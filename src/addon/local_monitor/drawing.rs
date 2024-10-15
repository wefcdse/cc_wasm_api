use crate::{
    addon::misc::{AsIfPixel, ColorId},
    debug::{self, show_str},
    eval::exec,
    prelude::LuaResult,
};

use super::LocalMonitor;

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
        create_str: bool,
    ) -> (String, usize) {
        draw.sort_by(|a, b| {
            (a.2.text_color, a.2.background_color).cmp(&(b.2.text_color, b.2.background_color))
        });
        let to_write = draw;

        // let gen_script_time = Instant::now();

        let mut write_script = String::new();
        write_script.reserve(10_000);
        let mut code_line = self.ext_script_set_color(&mut write_script, create_str, to_write[0].2);
        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        for (x, y, pix) in to_write.iter().copied() {
            let color = (pix.text_color, pix.background_color);
            if color != last_color {
                code_line += self.ext_script_set_color(&mut write_script, create_str, pix);
            }
            // let (s, c) = self.gen_script_write_txt(x, y, pix);
            // write_script += &s;
            // code_line += c;
            code_line += self.ext_script_set_cursor(&mut write_script, create_str, x, y);
            code_line += self.ext_script_write_char(&mut write_script, create_str, pix);
            last_color = color;
        }
        // show_str(&format!(
        //     "gen_script_time {}ms",
        //     gen_script_time.elapsed().as_secs_f32() * 1000.
        // ));
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
        create_str: bool,
    ) -> (String, usize) {
        if draw.is_empty() {
            return Default::default();
        }
        // let TTT = Instant::now();
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
            // color_cmp
        });
        // show_str(&format!("dsadsa {}", TTT.elapsed().as_secs_f32() * 1000.));

        let to_write = draw;
        let (mut write_script, mut code_line) = (String::new(), 0);

        let mut last_color = (to_write[0].2.text_color, to_write[0].2.background_color);
        {
            code_line += self.ext_script_set_color(&mut write_script, create_str, to_write[0].2);
        }
        let mut cursor_pos = (to_write[0].0, to_write[0].1);
        {
            code_line += self.ext_script_set_cursor(
                &mut write_script,
                create_str,
                to_write[0].0,
                to_write[0].1,
            );
        }

        let mut write_str = String::new();
        for (x, y, pix) in to_write.iter().copied() {
            let color = (pix.text_color, pix.background_color);
            let pos = (x, y);

            if color != last_color || pos != cursor_pos {
                code_line +=
                    self.ext_script_write_multi_char(&mut write_script, create_str, &write_str);
                write_str.clear();
            }

            if color != last_color {
                code_line += self.ext_script_set_color(&mut write_script, create_str, pix);
            }

            if pos != cursor_pos {
                code_line += self.ext_script_set_cursor(&mut write_script, create_str, x, y);
            }

            write_str.extend(pix.text().escape_debug());

            last_color = color;
            cursor_pos = (x + 1, y);
            if y == self.y() {
                cursor_pos = (1, y + 1)
            }
        }

        {
            // let (s, c) = self.gen_script_write_multi_char(&write_str);
            // write_script += &s;
            code_line +=
                self.ext_script_write_multi_char(&mut write_script, create_str, &write_str);
            write_str.clear();
        }
        (write_script, code_line)
    }

    pub(crate) fn gen_draw_opt_auto_clear(
        &self,
        draw: Vec<(usize, usize, AsIfPixel)>,
    ) -> (String, usize) {
        let (default_s, default_c) = self.gen_draw_opt_cursor_long_str(draw, true);

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
            let d = self.gen_draw_opt_cursor_long_str(cleared, true);
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
    pub(crate) fn gen_draw_opt_clear(&self, bg_color: ColorId) -> (String, usize) {
        // show_str("after iter");
        let clear_color = bg_color;
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

        let (clear_s, clear_c) = {
            let (mut s, mut c) = self.gen_script_clear(clear_color);
            let d = self.gen_draw_opt_cursor_long_str(cleared, true);
            s += &d.0;
            c += d.1;
            (s, c)
        };

        (clear_s, clear_c)
    }
}

// const BATCH: usize = 20000;
impl LocalMonitor {
    pub async fn sync(&mut self) -> LuaResult<usize> {
        // show_str(self.name());
        let to_write: Vec<(usize, usize, AsIfPixel)> = self.gen_nonsynced();

        if to_write.is_empty() {
            return Ok(0);
        }

        let changed_pix = to_write.len();
        let (write_script, code_line) = self.gen_draw_opt_cursor_long_str(to_write, true);

        exec(&write_script).await?;
        debug::show_str(&format!(
            "monitor [{}] draw code line: {}, changed pix: {}",
            self.name(),
            code_line,
            changed_pix
        ));

        self.last_sync = self.data.clone();
        Ok(changed_pix)
    }
    pub async fn sync_clear(&mut self, bg_color: ColorId) -> LuaResult<usize> {
        // show_str(self.name());
        let to_write: Vec<(usize, usize, AsIfPixel)> = self.gen_nonsynced();

        if to_write.is_empty() {
            return Ok(0);
        }

        let changed_pix = to_write.len();
        let (write_script, code_line) = self.gen_draw_opt_clear(bg_color);

        exec(&write_script).await?;
        debug::show_str(&format!(
            "monitor [{}] draw code line: {}, changed pix: {}",
            self.name(),
            code_line,
            changed_pix
        ));

        self.last_sync = self.data.clone();
        Ok(changed_pix)
    }
    /// # Safety
    /// the script must be execed
    pub unsafe fn sync_script(&mut self) -> (String, usize) {
        let to_write: Vec<(usize, usize, AsIfPixel)> = self.gen_nonsynced();

        if to_write.is_empty() {
            return (String::new(), 0);
        }
        let (write_script, code_line) = self.gen_draw_opt_cursor_long_str(to_write, true);

        self.last_sync = self.data.clone();
        (write_script, code_line)
    }
    /// # Safety
    /// the script must be execed
    pub unsafe fn sync_clear_script(&mut self, bg_color: ColorId) -> (String, usize) {
        let to_write: Vec<(usize, usize, AsIfPixel)> = self.gen_nonsynced();

        if to_write.is_empty() {
            return (String::new(), 0);
        }

        let (write_script, code_line) = self.gen_draw_opt_clear(bg_color);

        self.last_sync = self.data.clone();
        (write_script, code_line)
    }
}
