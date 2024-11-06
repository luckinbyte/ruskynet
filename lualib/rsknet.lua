print("require rsknet begin")
local rsknet = {}

local proto = {}
local rsknet = {
	PTYPE_RESPONSE = 1,
	PTYPE_LUA = 10,
}

rsknet.pack = function(...)
	local ret = rsknet_core_luapack(table.pack(...))
	print("rsknet pack", ret)
	return ret
end 
rsknet.unpack = function(...)
	rsknet_core_luaunpack(...)
end  

local session_id_coroutine = {}
local session_coroutine_id = {}
local session_coroutine_address = {}

local running_thread = nil
local init_thread = nil


local function yield_call(service, session)
	session_id_coroutine[session] = running_thread
	local succ, msg, sz = coroutine.yield "SUSPEND"
	return msg,sz
end

function rsknet.call(addr, typename, ...)
	local p = proto[typename]
	local session = rsknet_core_send(addr, p.id, 0, p.pack(...))
	return p.unpack(yield_call(addr, session))
end

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
	print("in rsknet dispatch_message", prototype, msg, session, source)

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
    pcall(raw_dispatch_message, ...)
end

local function co_create(f)
	local co = nil
	co = coroutine.create(function(...)
		f(...)
		while true do
			local session = session_coroutine_id[co]

			-- coroutine exit
			local address = session_coroutine_address[co]
			if address then
				session_coroutine_id[co] = nil
				session_coroutine_address[co] = nil
			end
			-- recycle co into pool
			f = nil
			coroutine_pool[#coroutine_pool+1] = co
			-- recv new main function f
			f = coroutine_yield "SUSPEND"
			f(coroutine_yield())
		end
	end)

	return co
end

function rsknet.timeout(ti, func)
	print("timeout fun")
	local session = tonumber(rsknet_core_command("TIMEOUT", ti))
	print("timeout session", session)
	local co = co_create(func)
	session_id_coroutine[session] = co
	return co
end

function rsknet.start(start_func)
	rsknet_core_callback(rsknet.dispatch_message)
	init_thread = rsknet.timeout(0, function() start_func() init_thread=nil end)
	-- init_thread = skynet.timeout(0, function()
	-- 	skynet.init_service(start_func)
	-- 	init_thread = nil
	-- end)
end

function rsknet.newservice(name, ...)
	-- return rsknet.call(".launcher", "lua", "LAUNCH", "snlua", name, ...)
	-- todo register ".launcher" to handle_id 2
	return rsknet.call(2, "lua", "LAUNCH", "snlua", name, ...)
end

-- regist dispatch fun
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

function rsknet.launch(...)
	local addr = rsknet_core_command("LAUNCH", table.concat({...}, " "))
	if addr then
		return tonumber(string.sub(addr , 2), 16)
	end
end

do
	local REG = rsknet.register_protocol
	REG {
		name = "lua",
		id = rsknet.PTYPE_LUA,
		pack = rsknet.pack,
		unpack = rsknet.unpack,
	}

	REG {
		name = "response",
		id = rsknet.PTYPE_RESPONSE,
	}

end

return rsknet