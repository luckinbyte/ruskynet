local rsknet = require "../lualib/rsknet"

rsknet.start(function() 
	local handle_id = rsknet_core_command("LAUNCH", table.concat({"snlua", "launcher"}," ") )
	print("LAUNCH handle_id", type(handle_id), handle_id)
	-- todo rsknet.name(".launcher", handle_id)

	local ret = rsknet.newservice "main" --todo config path
	print("start main service ret", ret)

	--rsknet.exit()
end)
