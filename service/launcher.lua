local rsknet = require "../lualib/rsknet"
local command = {}

local services = {}
--local instance = {}
--local launch_session = {}

local function launch_service(service, ...)
	local param = table.concat({...}, " ")
	local inst = rsknet.launch(service, param)  -- inst: handle_id
	local session = rsknet.context()
	--local response = rsknet.response()
	if inst then
		services[inst] = service .. " " .. param
		--instance[inst] = response
		--launch_session[inst] = session
	else
		--response(false)
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
		print("launch ret:", ret)
		rsknet.ret(rsknet.pack(ret))
		print("ret success")
    end
end)

rsknet.start(function() 
    print("launcher success") 
end)
