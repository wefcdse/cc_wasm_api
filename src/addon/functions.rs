use crate::{eval::exec, prelude::LuaResult};

use super::misc::{AsIfPixel, ColorId, Side};

pub async fn write_pix(x: i32, y: i32, pix: AsIfPixel, side: Side) -> LuaResult<()> {
    // return;
    let script = format!(
        "global.monitor{s}.setCursorPos({x}, {y})
    global.monitor{s}.setBackgroundColour({bc})
    global.monitor{s}.setTextColour({tc})
    global.monitor{s}.write(\"{txt}\")",
        s = side.name(),
        x = x + 1,
        y = y + 1,
        bc = pix.background_color.to_number(),
        tc = pix.text_color.to_number(),
        txt = pix.text()
    );
    // show_str(&script);
    exec(&script).await
}

pub async fn init_monitor(side: Side) -> LuaResult<()> {
    exec(&format!(
        "global.monitor{} = peripheral.wrap(\"top\")",
        side.name()
    ))
    .await
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
