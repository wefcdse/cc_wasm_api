use crate::{
    addon::misc::{AsIfPixel, ColorId, Side},
    eval::{eval, exec},
    prelude::LuaResult,
    utils::Number,
};

use super::LocalMonitor;
impl LocalMonitor {
    pub(crate) fn gen_name(side: Side) -> String {
        format!("monitor_local_{s}", s = side.name())
    }
}

impl LocalMonitor {
    pub async fn init(&mut self, side: Side) -> LuaResult<()> {
        let inited = Self::new_inited(side).await?;
        *self = inited;
        Ok(())
    }
    pub async fn new_inited(side: Side) -> LuaResult<Self> {
        exec(&Self::gen_script_init_monitor(side)).await?;
        let (x, y): (Number, Number) = eval(&format!(
            "return {}.getSize()",
            LocalMonitor::gen_name(side)
        ))
        .await?;
        let (x, y) = (x.to_i32() as usize, y.to_i32() as usize);

        let mut new_self = Self::new(x, y, AsIfPixel::default(), side);
        new_self
            .clear(AsIfPixel::default().background_color)
            .await?;

        Ok(new_self)
    }
    /// returns if resized
    pub async fn sync_size(&mut self) -> LuaResult<bool> {
        let (x, y): (Number, Number) = eval(&format!("return {}.getSize()", self.name())).await?;
        let (x, y) = (x.to_i32() as usize, y.to_i32() as usize);
        if self.size() == (x, y) {
            return Ok(false);
        }
        self.resize(x, y, AsIfPixel::default());
        // let mut new_self = Self::new(x, y, AsIfPixel::default(), side);
        self.clear(AsIfPixel::default().background_color).await?;

        Ok(true)
    }

    pub async fn clear(&mut self, color: ColorId) -> LuaResult<()> {
        exec(&self.gen_script_clear(color).0).await?;
        self.data.iter_mut().for_each(|(_, pix)| {
            *pix = AsIfPixel::colored_whitespace(color);
        });
        self.last_sync = self.data.clone();
        Ok(())
    }
}
