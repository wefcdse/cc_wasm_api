use crate::{
    eval::eval,
    prelude::{Importable, LuaResult},
};
pub async fn get_args<O>() -> LuaResult<O>
where
    O: Importable + Unpin + 'static,
{
    eval("return unpack(args)").await
}
