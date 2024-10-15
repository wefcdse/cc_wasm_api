use crate::{
    addon::misc::{AsIfPixel, ColorId, Side},
    eval::{eval, exec},
    prelude::LuaResult,
    utils::Number,
};

use super::LocalMonitor;
impl LocalMonitor {
    pub(crate) fn gen_name(init_method: InitMethod<'_>) -> String {
        match init_method {
            InitMethod::Remote { side, name } => {
                format!("monitor_remote_{s}_{n}", s = side.name(), n = name)
            }
            InitMethod::Local(side) => format!("monitor_local_{s}", s = side.name()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum InitMethod<'a> {
    Remote { side: Side, name: &'a str },
    Local(Side),
}
impl<'a> From<Side> for InitMethod<'a> {
    fn from(value: Side) -> Self {
        Self::Local(value)
    }
}
impl<'a> From<(Side, &'a str)> for InitMethod<'a> {
    fn from(value: (Side, &'a str)) -> Self {
        Self::Remote {
            side: value.0,
            name: value.1,
        }
    }
}
impl<'a> From<(Side, &'a String)> for InitMethod<'a> {
    fn from(value: (Side, &'a String)) -> Self {
        Self::Remote {
            side: value.0,
            name: value.1,
        }
    }
}

impl LocalMonitor {
    pub async fn init(&mut self, init_method: impl Into<InitMethod<'_>>) -> LuaResult<()> {
        let inited = Self::new_inited(init_method.into()).await?;
        *self = inited;
        Ok(())
    }
    pub async fn new_inited(init_method: impl Into<InitMethod<'_>>) -> LuaResult<Self> {
        let init_method = init_method.into();
        exec(&Self::gen_script_init_monitor(init_method)).await?;
        let (x, y): (Number, Number) = eval(&format!(
            "return {}.getSize()",
            LocalMonitor::gen_name(init_method)
        ))
        .await?;
        let (x, y) = (x.to_i32() as usize, y.to_i32() as usize);

        let mut new_self = Self::new(x, y, AsIfPixel::default(), init_method);
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
