use crate::errors::*;
use crate::engine::ctx::State;
use crate::hlua;
use chrono::{NaiveDateTime, Utc};
use std::sync::Arc;


const SN0INT_DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

// TODO: deprecate this function
pub fn datetime(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("datetime", hlua::function0(move || -> String {
        let now = Utc::now().naive_utc();
        now.format(SN0INT_DATETIME_FORMAT)
           .to_string()
    }))
}

pub fn sn0int_time(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("sn0int_time", hlua::function0(move || -> String {
        let now = Utc::now().naive_utc();
        now.format(SN0INT_DATETIME_FORMAT)
           .to_string()
    }))
}

pub fn sn0int_time_from(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("sn0int_time_from", hlua::function1(move |time: i32| -> Result<String> {
        let time = NaiveDateTime::from_timestamp_opt(time.into(), 0)
            .ok_or_else(|| format_err!("Failed to get time from timestamp"))?;
        let time = time.format(SN0INT_DATETIME_FORMAT).to_string();
        Ok(time)
    }))
}

pub fn strftime(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("strftime", hlua::function2(move |format: String, time: i32| -> Result<String> {
        let time = NaiveDateTime::from_timestamp_opt(time.into(), 0)
            .ok_or_else(|| format_err!("Failed to get time from timestamp"))?;
        let time = time.format(&format).to_string();
        Ok(time)
    }))
}

pub fn strptime(lua: &mut hlua::Lua, state: Arc<dyn State>) {
    lua.set("strptime", hlua::function2(move |format: String, time: String| -> Result<i32> {
        let datetime = NaiveDateTime::parse_from_str(&time, &format)
            .map_err(|err| state.set_error(Error::from(err)))?;
        Ok(datetime.timestamp() as i32)
    }))
}

pub fn time_unix(lua: &mut hlua::Lua, _: Arc<dyn State>) {
    lua.set("time_unix", hlua::function0(move || -> i32 {
        let now = Utc::now().naive_utc();
        now.timestamp() as i32
    }))
}

#[cfg(test)]
mod tests {
    use crate::engine::ctx::Script;

    // TODO: deprecate this function
    #[test]
    fn verify_datetime() {
        let script = Script::load_unchecked(r#"
        function run()
            now = datetime()
            print(now)
            if regex_find("^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}$", now) == nil then
                return 'invalid date'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_sn0int_time() {
        let script = Script::load_unchecked(r#"
        function run()
            now = sn0int_time()
            print(now)
            if regex_find("^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}$", now) == nil then
                return 'invalid date'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_sn0int_time_from() {
        let script = Script::load_unchecked(r#"
        function run()
            t = sn0int_time_from(1567931337)
            if t ~= '2019-09-08T08:28:57' then
                return 'invalid: ' .. t
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_strftime() {
        let script = Script::load_unchecked(r#"
        function run()
            t = strftime('%d/%m/%Y %H:%M', 1558584994)
            print(t)
            if t ~= '23/05/2019 04:16' then
                return 'wrong time'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");

    }

    #[test]
    fn verify_strptime() {
        let script = Script::load_unchecked(r#"
        function run()
            t = strptime('%d/%m/%Y %H:%M', '23/05/2019 04:16')
            print(t)
            if t ~= 1558584960 then
                return 'wrong time'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }

    #[test]
    fn verify_time_unix() {
        let script = Script::load_unchecked(r#"
        function run()
            now = time_unix()
            print(now)
            if now <= 1558584994 then
                return 'time went backwards'
            end
        end
        "#).expect("Failed to load script");
        script.test().expect("Script failed");
    }
}
