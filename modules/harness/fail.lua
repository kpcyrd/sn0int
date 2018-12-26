-- Description: Test error handling
-- Version: 0.1.0
-- License: GPL-3.0

function run()
    error('This is a non fatal error')
    return "This is an error: " .. 123
end
