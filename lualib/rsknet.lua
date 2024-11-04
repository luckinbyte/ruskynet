local rsknet = {}

local proto = {}

local session_id_coroutine = {}
local session_coroutine_id = {}
local session_coroutine_address = {}

local running_thread = nil

local function coroutine_resume(co, ...)
	running_thread = co
	return coroutine.resume(co, ...)
end

function rsknet.register_protocol(class)
	local name = class.name
	local id = class.id
	proto[name] = class
	proto[id] = class
end

function rsknet.dispatch(typename, func)
	local p = proto[typename]
	if func then
		local ret = p.dispatch
		p.dispatch = func
		return ret
	else
		return p and p.dispatch
	end
end

function suspend(co, result, command)
	if not result then
        -- todo
	end
	if command == "SUSPEND" then
		return dispatch_wakeup()
	elseif command == "QUIT" then
		coroutine.close(co)
		return
	elseif command == nil then
		return
	else
		error("Unknown command : " .. command .. "\n" .. traceback(co))
	end
end

local function raw_dispatch_message(prototype, msg, session, source)
    if prototype == 1 then
        local co = session_id_coroutine[session]
        if co == "BREAK" then
            session_id_coroutine[session] = nil
        else
            session_id_coroutine[session] = nil
            suspend(co, coroutine_resume(co, true, msg, session))
        end
    else
        local p = proto[prototype]    
		local f = p.dispatch  
        local co = co_create(f)   
        session_coroutine_id[co] = session
        session_coroutine_address[co] = source
        suspend(co, coroutine_resume(co, session, source, p.unpack(msg)))
    end
end

function rsknet.dispatch_message(...)
	print("in rsknet dispatch_message", ...)
    pcall(raw_dispatch_message, ...)
end

function rsknet.start(start_func)
	rsknet_core_callback(rsknet.dispatch_message)

	-- init_thread = skynet.timeout(0, function()
	-- 	skynet.init_service(start_func)
	-- 	init_thread = nil
	-- end)
end

return rsknet