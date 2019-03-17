use crate::errors::*;

use crate::engine::ctx::State;
use crate::gfx;
use crate::json::LuaJsonValue;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


pub fn img_exif(lua: &mut hlua::Lua, state: Arc<State>) {
    lua.set("img_exif", hlua::function1(move |blob: String| -> Result<AnyLuaValue> {
        let img = state.get_blob(&blob)
            .map_err(|err| state.set_error(err))?;

        let location = gfx::exif::gps(&img.bytes)
            .map_err(|err| state.set_error(err))?;

        let location = serde_json::to_value(location)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(location).into())
    }))
}


#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    #[test]
    #[ignore]
    fn verify_img_exif() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            url = 'https://raw.githubusercontent.com/ianare/exif-samples/master/jpg/gps/DSCN0012.jpg'
            req = http_request(session, 'GET', url, {
                into_blob=true,
            })
            r = http_send(req)
            if last_err() then return end
            print(r)
            if r['status'] ~= 200 then return 'wrong status code' end

            location = img_exif(r['blob'])
            print(location)

            if location == nil then
                return 'location lookup failed'
            end

            if location['latitude'] ~= 43.467157 then
                return 'latitude incorrect'
            end

            if location['longitude'] ~= 11.885395 then
                return 'longitude incorrect'
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }
}
