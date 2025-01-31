use fast_number::NUM_MAP;

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
mod fast_number {
    include!(concat!(env!("OUT_DIR"), "/num_map.rs"));
}
impl LocalMonitor {
    pub(crate) fn ext_script_set_color(
        &self,
        out: &mut String,
        create_str: bool,
        pix: AsIfPixel,
    ) -> usize {
        if create_str {
            // return;
            out.push_str("global.");
            out.push_str(self.name());
            out.push_str(".setBackgroundColour(");
            out.push_str(pix.background_color.to_str());
            out.push_str(")\nglobal.");
            out.push_str(self.name());
            out.push_str(".setTextColour(");
            out.push_str(pix.text_color.to_str());
            out.push_str(")\n");
            //     let script = format!(
            //         "global.{n}.setBackgroundColour({bc})
            // global.{n}.setTextColour({tc})
            // \n",
            //         n = self.name(),
            //         bc = pix.background_color.to_number(),
            //         tc = pix.text_color.to_number(),
            //     );)
        }
        2
    }
    pub(crate) fn ext_script_set_cursor(
        &self,
        out: &mut String,
        create_str: bool,
        x: usize,
        y: usize,
    ) -> usize {
        if create_str {
            out.push_str("global.");
            out.push_str(self.name());
            out.push_str(".setCursorPos(");
            out.push_str(NUM_MAP[x + 1]);
            out.push_str(", ");
            out.push_str(NUM_MAP[y + 1]);
            out.push_str(")\n");
        }
        // // return;
        // let script = format!(
        //     "global.{n}.setCursorPos({x}, {y})\n",
        //     n = self.name(),
        //     x = x + 1,
        //     y = y + 1,
        // );
        1
    }
    pub(crate) fn ext_script_write_multi_char(
        &self,
        out: &mut String,
        create_str: bool,
        txt: &str,
    ) -> usize {
        if create_str {
            out.push_str("global.");
            out.push_str(self.name());
            out.push_str(".write(\"");
            out.push_str(txt);
            out.push_str("\")\n");
        }
        1
    }
    pub(crate) fn ext_script_write_char(
        &self,
        out: &mut String,
        create_str: bool,
        pix: AsIfPixel,
    ) -> usize {
        if create_str {
            out.push_str("global.");
            out.push_str(self.name());
            out.push_str(".write('");
            out.push(pix.text());
            out.push_str("')\n");
        }
        // // return;
        // let script = format!(
        //     "global.{n}.write({txt:?})\n",
        //     n = self.name(),
        //     txt = pix.text()
        // );
        1
    }
}

#[allow(dead_code)]
impl LocalMonitor {
    pub(crate) fn gen_script_set_color(&self, pix: AsIfPixel) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setBackgroundColour({bc})\nglobal.{n}.setTextColour({tc})\n",
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
            "global.{n}.setCursorPos({x}, {y})\nglobal.{n}.write({txt:?})\n",
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
            "global.{n}.setBackgroundColour({bc})\nglobal.{n}.clear()",
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
                "{func}\nglobal.{n} = wrap_remote({s:?}, {rn:?})",
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

impl LocalMonitor {
    pub(crate) fn gen_script_set_palette(&self, clr: ColorId, target: u32) -> (String, usize) {
        // return;
        let script = format!(
            "global.{n}.setPaletteColour({clr}, {t})\n",
            n = self.name(),
            clr = clr.to_number(),
            t = target
        );
        (script, 1)
    }
}
