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

rsknet.start(function() 
	print("in bootstrap fun")
	local handle_id = rsknet_core_command("LAUNCH", table.concat({"snlua", "launcher"}," ") )
	-- todo rsknet.name(".launcher", handle_id)

	rsknet.newservice "service_mgr"

	--rsknet.exit()
end)
