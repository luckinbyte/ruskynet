local rsknet = require "../lualib/rsknet"
local command = {}

local function launch_service(service, ...)
    print("in launch_service")
	local param = table.concat({...}, " ")
	local inst = rsknet.launch(service, param)
	local session = rsknet.context()
	local response = rsknet.response()
	if inst then
		services[inst] = service .. " " .. param
		instance[inst] = response
		launch_session[inst] = session
	else
		response(false)
		return
	end
	return inst
end

function command.LAUNCH(_, service, ...)
	launch_service(service, ...)
	--return NORET
    return 1
end

rsknet.dispatch("lua", function(session, address, cmd , ...)
	local f = command[cmd]
	if f then
		local ret = f(address, ...)
		rsknet.ret(rsknet.pack(ret))
    end
end)

rsknet.start(function() 
    print("launcher success") 
end)
