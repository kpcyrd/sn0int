-- Description: Reproduce a rust stdlib panic
-- Version: 0.1.0

function run()
    --[[
    See for details:
    https://github.com/rust-lang/rust/issues/54267
    https://github.com/rust-lang/rust/issues/39364
    ]]--
    sleep(10)
end
