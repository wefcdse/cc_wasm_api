use super::{
    super::misc::{AsIfPixel, ColorId},
    initing::InitMethod,
    LocalMonitor,
};

// pub fn _write_pix(x: usize, y: usize, pix: AsIfPixel, side: Side) -> String {
//     // return;
//     let script = format!(
//         "global.monitor{s}.setCursorPos({x}, {y})
//     global.monitor{s}.setBackgroundColour({bc})
//     global.monitor{s}.setTextColour({tc})
//     global.monitor{s}.write(\"{txt}\") \n",
//         s = side.name(),
//         x = x + 1,
//         y = y + 1,
//         bc = pix.background_color.to_number(),
//         tc = pix.text_color.to_number(),
//         txt = pix.text()
//     );
//     script
// }
#[allow(dead_code)]
impl LocalMonitor {
    pub(crate) fn gen_script_set_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setBackgroundColour({bc})
    global.{n}.setTextColour({tc})
    \n",
            n = self.name(),
            bc = pix.background_color.to_number(),
            tc = pix.text_color.to_number(),
        );
        (script, 2)
    }
    pub(crate) fn gen_script_set_txt_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setTextColour({tc})\n",
            n = self.name(),
            tc = pix.text_color.to_number(),
        );
        (script, 1)
    }
    pub(crate) fn gen_script_set_bg_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setBackgroundColour({bc})\n",
            n = self.name(),
            bc = pix.background_color.to_number(),
        );
        (script, 1)
    }
    pub(crate) fn gen_script_set_cursor(&self, x: usize, y: usize) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setCursorPos({x}, {y})\n",
            n = self.name(),
            x = x + 1,
            y = y + 1,
        );
        (script, 1)
    }
    pub(crate) fn gen_script_write_multi_char(&self, txt: &str) -> (String, usize) {
        // show_str(txt);
        // return;
        let script = format!("global.{n}.write({txt:?})\n", n = self.name());
        (script, 1)
    }
    pub(crate) fn gen_script_write_char(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.write({txt:?})\n",
            n = self.name(),
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
            "global.{n}.setCursorPos({x}, {y})
        global.{n}.write({txt:?}) \n",
            n = self.name(),
            x = x + 1,
            y = y + 1,
            txt = pix.text()
        );
        (script, 2)
    }

    pub(crate) fn gen_script_clear(&self, color: ColorId) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setBackgroundColour({bc})\n global.{n}.clear()",
            n = self.name(),
            bc = color.to_number(),
        );
        (script, 2)
    }
}

impl LocalMonitor {
    pub fn gen_script_init_monitor(init_method: InitMethod<'_>) -> String {
        match init_method {
            InitMethod::Remote { side, name } => format!(
                "{func}\n global.{n} = wrap_remote({s:?}, {rn:?})",
                func = include_str!("wrap_peri.lua"),
                s = side.name(),
                n = LocalMonitor::gen_name(init_method),
                rn = name
            ),
            InitMethod::Local(side) => format!(
                "global.{n} = peripheral.wrap(\"{s}\")",
                s = side.name(),
                n = LocalMonitor::gen_name(init_method)
            ),
        }
        // if is_remote {
        //     let script = format!(
        //         "{func}\nglobal.{n} = peripheral.wrap(\"{s}\")",
        //         func = include_str!("wrap_peri.lua"),
        //         s = side.name(),
        //         n = LocalMonitor::gen_name(side)
        //     );

        //     script
        // } else {
        //     let script = format!(
        //         "global.{n} = peripheral.wrap(\"{s}\")",
        //         s = side.name(),
        //         n = LocalMonitor::gen_name(side)
        //     );

        //     script
        // }
    }
}
