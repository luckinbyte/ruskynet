local rsknet = require "../lualib/rsknet"

-- rsknet.start(function()
-- 	local launcher = rsknet.launch("snlua","launcher")
-- 	rsknet.name(".launcher", launcher)

--     local ok, slave = pcall(rsknet.newservice, "cdummy")
--     rsknet.name(".cslave", slave)

--     rsknet.newservice "service_mgr"

-- 	pcall(rsknet.newservice, rsknet.getenv "start" or "main")
-- 	rsknet.exit()
-- end)

print("in bootstrap lua begin")
rsknet.start(function() end)
print("in bootstrap lua end")
