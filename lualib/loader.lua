local f,msg = loadfile("./service/"..select(1, ...))
--_G.require = require("rskynet_require").require
if f then
    f()
else
    local f,msg = loadfile("./examples/"..select(1, ...))
    f()
end