use crate::{eval::exec, prelude::LuaResult};

use super::{
    super::misc::{AsIfPixel, ColorId, Side},
    LocalMonitor,
};

pub fn _write_pix(x: usize, y: usize, pix: AsIfPixel, side: Side) -> String {
    // return;
    let script = format!(
        "global.monitor{s}.setCursorPos({x}, {y})
    global.monitor{s}.setBackgroundColour({bc})
    global.monitor{s}.setTextColour({tc})
    global.monitor{s}.write(\"{txt}\") \n",
        s = side.name(),
        x = x + 1,
        y = y + 1,
        bc = pix.background_color.to_number(),
        tc = pix.text_color.to_number(),
        txt = pix.text()
    );
    script
}
#[allow(dead_code)]
impl LocalMonitor {
    pub(crate) fn gen_script_set_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.setBackgroundColour({bc})
    global.monitor{s}.setTextColour({tc})
    \n",
            s = self.side.name(),
            bc = pix.background_color.to_number(),
            tc = pix.text_color.to_number(),
        );
        (script, 2)
    }
    pub(crate) fn gen_script_set_txt_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.setTextColour({tc})\n",
            s = self.side.name(),
            tc = pix.text_color.to_number(),
        );
        (script, 1)
    }
    pub(crate) fn gen_script_set_bg_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.setBackgroundColour({bc})\n",
            s = self.side.name(),
            bc = pix.background_color.to_number(),
        );
        (script, 1)
    }
    pub(crate) fn gen_script_set_cursor(&self, x: usize, y: usize) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.setCursorPos({x}, {y})\n",
            s = self.side.name(),
            x = x + 1,
            y = y + 1,
        );
        (script, 1)
    }
    pub(crate) fn gen_script_write_multi_char(&self, txt: &str) -> (String, usize) {
        // show_str(txt);
        // return;
        let script = format!("global.monitor{s}.write({txt:?})\n", s = self.side.name(),);
        (script, 1)
    }
    pub(crate) fn gen_script_write_char(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.write({txt:?})\n",
            s = self.side.name(),
            txt = pix.text()
        );
        (script, 1)
    }
    pub(crate) fn gen_script_write_txt(
        &self,
        x: usize,
        y: usize,
        pix: AsIfPixel,
    ) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.setCursorPos({x}, {y})
    global.monitor{s}.write(\"{txt}\") \n",
            s = self.side.name(),
            x = x + 1,
            y = y + 1,
            txt = pix.text()
        );
        (script, 2)
    }

    pub(crate) fn gen_script_clear(&self, color: ColorId) -> (String, usize) {
        // return;
        let script = format!(
            "global.monitor{s}.setBackgroundColour({bc})\n global.monitor{s}.clear()",
            s = self.side.name(),
            bc = color.to_number(),
        );
        (script, 2)
    }
}

pub async fn init_monitor(side: Side) -> LuaResult<()> {
    let script = format!(
        "global.monitor{s} = peripheral.wrap(\"{s}\")",
        s = side.name()
    );

    exec(&script).await
}

pub async fn clear(color: ColorId, side: Side) -> LuaResult<()> {
    // return;
    let script = format!(
        "   global.monitor{s}.setBackgroundColour({bc})
            global.monitor{s}.clear()",
        s = side.name(),
        bc = color.to_number(),
    );
    // show_str(&script);
    exec(&script).await
}
