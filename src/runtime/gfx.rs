use crate::errors::*;
use serde::Serialize;

use crate::engine::ctx::State;
use crate::gfx;
use crate::json::LuaJsonValue;
use crate::hlua::{self, AnyLuaValue};
use std::sync::Arc;


#[derive(Debug, Serialize)]
pub struct ImageData<'a> {
    mime: &'a str,
    width: u32,
    height: u32,
}

pub fn img_load(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("img_load", hlua::function1(move |blob: String| -> Result<AnyLuaValue> {
        let img = state.get_blob(&blob)
            .map_err(|err| state.set_error(err))?;

        let img = gfx::load(&img.bytes)
            .map_err(|err| state.set_error(err))?;

        let data = ImageData {
            mime: img.mime(),
            width: img.width(),
            height: img.height(),
        };

        let data = serde_json::to_value(data)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(data).into())
    }))
}

pub fn img_ahash(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("img_ahash", hlua::function1(move |blob: String| -> Result<String> {
        let img = state.get_blob(&blob)
            .map_err(|err| state.set_error(err))?;

        let img = gfx::load(&img.bytes)
            .map_err(|err| state.set_error(err))?;

        let hash = img.perception_hash(gfx::HashAlg::Mean);

        Ok(hash)
    }))
}

pub fn img_dhash(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("img_dhash", hlua::function1(move |blob: String| -> Result<String> {
        let img = state.get_blob(&blob)
            .map_err(|err| state.set_error(err))?;

        let img = gfx::load(&img.bytes)
            .map_err(|err| state.set_error(err))?;

        let hash = img.perception_hash(gfx::HashAlg::Gradient);

        Ok(hash)
    }))
}

pub fn img_phash(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("img_phash", hlua::function1(move |blob: String| -> Result<String> {
        let img = state.get_blob(&blob)
            .map_err(|err| state.set_error(err))?;

        let img = gfx::load(&img.bytes)
            .map_err(|err| state.set_error(err))?;

        let hash = img.perception_hash(gfx::HashAlg::Median);

        Ok(hash)
    }))
}

#[derive(Debug, Serialize)]
pub struct Nudity {
    nude: bool,
    skin_percent: f64,
    score: f64,
}

pub fn img_nudity(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("img_nudity", hlua::function1(move |blob: String| -> Result<AnyLuaValue> {
        let img = state.get_blob(&blob)
            .map_err(|err| state.set_error(err))?;

        let img = gfx::load(&img.bytes)
            .map_err(|err| state.set_error(err))?;

        let nudity = nude::scan(img.as_ref()).analyse();

        let nudity = Nudity {
            nude: nudity.nude,
            skin_percent: nudity.skin_percent,
            score: nudity.score(),
        };

        let nudity = serde_json::to_value(nudity)
            .map_err(|e| state.set_error(e.into()))?;

        Ok(LuaJsonValue::from(nudity).into())
    }))
}

pub fn img_exif(lua: &mut hlua::Lua, state: Arc<dyn State>) {
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
    fn verify_img_load() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, 'GET', 'https://www.kernel.org/theme/images/logos/tux.png', {
                into_blob=true,
            })
            r = http_fetch(req)
            if last_err() then return end

            img = img_load(r['blob'])

            if img['mime'] ~= 'image/png' then
                return 'mime incorrect: ' .. img['mime']
            end
            if img['width'] ~= 104 then
                return 'width incorrect: ' .. img['width']
            end
            if img['height'] ~= 120 then
                return 'height incorrect: ' .. img['height']
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

    // TODO: find better picture
    #[test]
    #[ignore]
    fn verify_img_nudity() {
        let script = Script::load_unchecked(r#"
        function run()
            session = http_mksession()
            req = http_request(session, 'GET', 'https://www.kernel.org/theme/images/logos/tux.png', {
                into_blob=true,
            })
            r = http_fetch(req)
            if last_err() then return end

            nudity = img_nudity(r['blob'])

            if nudity['nude'] ~= false then
                return 'nude incorrect: ' .. nudity['nude']
            end
            if nudity['skin_percent'] ~= 0 then
                return 'skin_percent incorrect: ' .. nudity['skin_percent']
            end
            if nudity['score'] ~= 0 then
                return 'score incorrect: ' .. nudity['score']
            end
        end
        "#).expect("failed to load script");
        script.test().expect("Script failed");
    }

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
            r = http_fetch(req)
            if last_err() then return end
            print(r)

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
