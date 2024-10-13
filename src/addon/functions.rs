use crate::{eval::exec, prelude::LuaResult};

use super::misc::{AsIfPixel, ColorId, Side};

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

pub fn set_color(pix: AsIfPixel, side: Side) -> String {
    // return;
    let script = format!(
        "global.monitor{s}.setBackgroundColour({bc})
    global.monitor{s}.setTextColour({tc})
    \n",
        s = side.name(),
        bc = pix.background_color.to_number(),
        tc = pix.text_color.to_number(),
    );
    script
}

pub fn write_txt(x: usize, y: usize, pix: AsIfPixel, side: Side) -> String {
    // return;
    let script = format!(
        "global.monitor{s}.setCursorPos({x}, {y})
    global.monitor{s}.write(\"{txt}\") \n",
        s = side.name(),
        x = x + 1,
        y = y + 1,
        txt = pix.text()
    );
    script
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
