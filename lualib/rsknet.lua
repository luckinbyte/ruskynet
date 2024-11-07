local rsknet = {}

local proto = {}
local rsknet = {
	PTYPE_RESPONSE = 1,
	PTYPE_LUA = 10,
}

rsknet.pack = function(...)
	local ret = rsknet_core_luapack(table.pack(...))
	return ret
end 
rsknet.unpack = function(...)
	rsknet_core_luaunpack(...)
end  

local session_id_coroutine = {}  -- co 2 s_id
local session_coroutine_id = {}	-- s_id 2 co
local session_coroutine_address = {}	-- s_id 2 handle
local unresponse = {}

local running_thread = nil
local init_thread = nil


local function yield_call(service, session)
	session_id_coroutine[session] = running_thread
	local succ, msg, sz = coroutine.yield "SUSPEND"
	return msg,sz
end

local function co_create(f)
	local co = nil
	co = coroutine.create(function(...)
		f(...)
	end)
	return co
end

function rsknet.call(addr, typename, ...)
	local p = proto[typename]
	local session = rsknet_core_send(addr, p.id, 0, p.pack(...))
	print(string.format("rsknet.call hand:%s session:%s running_thread:%s", HANDLE_ID, session, running_thread), addr, typename, ...)
	return p.unpack(yield_call(addr, session))
end

local function coroutine_resume(co, ...)
	running_thread = co
	--print("coroutine_resume:", co, ...)
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

local function temp_msg_print(msg)	
	local result = {}
	for _, v in ipairs(msg) do
		local c = string.char(v)
		if c == '[' then
			result[#result + 1] = '{'
		elseif c == ']' then
			result[#result + 1] = '}'
		else
			result[#result + 1] = c
		end
	end
	if not next(result) then
		print("!!!", type(result), #table)
		return nil
	end
	return table.concat(result)
end

local function raw_dispatch_message(prototype, msg, session, source)
	print(string.format("raw_dispatch_message hand:%s ptype:%s msg:%s session:%s source:%s", HANDLE_ID, prototype, temp_msg_print(msg), session, source))

    if prototype == 1 then
        local co = session_id_coroutine[session]
        session_id_coroutine[session] = nil
        coroutine_resume(co, true, msg, session)
    else
        local p = proto[prototype]    
		local f = p.dispatch  
        local co = co_create(f)   
        session_coroutine_id[co] = session
        session_coroutine_address[co] = source
        coroutine_resume(co, session, source, table.unpack(p.unpack(msg)))
    end
end

function rsknet.dispatch_message(...)
    pcall(raw_dispatch_message, ...)
end

function rsknet.timeout(ti, func)
	local session = tonumber(rsknet_core_command("TIMEOUT", ti))
	local co = co_create(func)
	session_id_coroutine[session] = co
	return co
end

function rsknet.start(start_func)
	rsknet_core_callback(rsknet.dispatch_message)
	rsknet.timeout(0, function() start_func() end)
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
		return tonumber(addr)
	end
end

do
	local REG = rsknet.register_protocol
	REG {
		name = "lua",
		id = rsknet.PTYPE_LUA,
		pack = rsknet.pack,
		unpack = --todo  rsknet.unpack,
			function(str_table)	
				local result = {"return "}
				for _, v in ipairs(str_table) do
					local c = string.char(v)
					if c == '[' then
						result[#result + 1] = '{'
					elseif c == ']' then
						result[#result + 1] = '}'
					else
						result[#result + 1] = c
					end
				end
				local str = table.concat(result)
				return load(str)()
			end
	}

	REG {
		name = "response",
		id = rsknet.PTYPE_RESPONSE,
	}

end


-- function rsknet.response(pack)
-- 	pack = pack or rsknet.pack

-- 	session_coroutine_id[running_thread] = nil
-- 	local co_address = session_coroutine_address[running_thread]
-- 	if co_session == 0 then
-- 		return function() end
-- 	end
-- 	local function response(ok, ...)
-- 		local ret
-- 		if unresponse[response] then
-- 			if ok then
-- 				ret = rsknet_core_send(co_address, rsknet.PTYPE_RESPONSE, co_session, pack(...))
-- 			else
-- 				--ret = c.send(co_address, rsknet.PTYPE_ERROR, co_session, "")
-- 			end
-- 			unresponse[response] = nil
-- 			ret = ret ~= nil
-- 		else
-- 			ret = false
-- 		end
-- 		pack = nil
-- 		return ret
-- 	end
-- 	unresponse[response] = co_address
-- 	return response
-- end

function rsknet.context()
	local co_session = session_coroutine_id[running_thread]
	local co_address = session_coroutine_address[running_thread]
	return co_session, co_address
end


function rsknet.ret(msg)
	msg = msg or ""
	local co_session = session_coroutine_id[running_thread]

	session_coroutine_id[running_thread] = nil
	local co_address = session_coroutine_address[running_thread]
	local ret = rsknet_core_send(co_address, rsknet.PTYPE_RESPONSE, co_session, msg)
	if ret then
		return true
	elseif ret == false then
		-- If the package is too large, returns false. so we should report error back
		-- c.send(co_address, skynet.PTYPE_ERROR, co_session, "")
	end
	return false
end

return rsknet