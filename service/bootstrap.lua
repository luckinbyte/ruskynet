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
	rsknet_core_command("LAUNCH", table.concat({"snlua", "launcher"}," ") )
	-- rsknet.dispatch("lua", function(session, source, cmd, subcmd, ...)
	-- 	if cmd == "socket" then
	-- 		local f = SOCKET[subcmd]
	-- 		f(...)
	-- 	else
	-- 		local f = CMD[cmd]
	-- 		skynet.ret(skynet.pack(f(subcmd, ...)))
	-- 	end
	-- end)
end)
