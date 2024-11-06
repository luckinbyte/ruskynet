local rsknet = require "../lualib/rsknet"

rsknet.start(function() 
	print("in bootstrap fun")
	local handle_id = rsknet_core_command("LAUNCH", table.concat({"snlua", "launcher"}," ") )
	-- todo rsknet.name(".launcher", handle_id)

	local ret = rsknet.newservice "main" --todo config path
	print("start main service ret", ret)

	--rsknet.exit()
end)
