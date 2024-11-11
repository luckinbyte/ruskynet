local rsknet = require "../lualib/rsknet"

local socket = {}	
local socket_pool = {}
local socket_message = {}

local function suspend(s)
	s.co = coroutine.running()
	--skynet.wait(s.co)
    coroutine.yield "SUSPEND"
end

local function wakeup(s)
	local co = s.co
	if co then
		s.co = nil
		coroutine.resume(co)
	end
end

local function connect(id, func)
	local s = {
		id = id,
		connected = false,
		connecting = true,
		read_required = false,
		co = false,
		callback = func,
		protocol = "TCP",
	}
	local s2 = socket_pool[id]
	socket_pool[id] = s
	suspend(s)
	return id
end

-- RSKNET_SOCKET_TYPE_DATA = 1
socket_message[1] = function(id, size, data)
	local s = socket_pool[id]
	wakeup(s)
end

-- RSKNET_SOCKET_TYPE_CONNECT = 2
socket_message[2] = function(id, ud, addr)
	local s = socket_pool[id]
	if s == nil then
		return
	end
	if not s.connected then	
		if s.listen then
			s.addr = addr
			s.port = ud
		end
		s.connected = true
		wakeup(s)
	end
end

-- RSKNET_SOCKET_TYPE_ACCEPT = 4
socket_message[4] = function(id, newid, addr)
	local s = socket_pool[id]
	s.callback(newid, addr)
end

rsknet.register_protocol {
	name = "socket",
	id = rsknet.PTYPE_SOCKET,	-- PTYPE_SOCKET = 6
	unpack = function(str_table)
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
        --print("socket unpack", table.concat(result))
        local str = table.concat(result)
        return load(str)()
    end,
	dispatch = function (_, _, t, ...)
        print("in socket dispatch", t, ...)
		socket_message[t](...)
	end
}

function socket.listen(host, port)
    local id = rsknet_socket_listen("0.0.0.0", 4276)
	local s = {
		id = id,
		connected = false,
		listen = true,
	}
	socket_pool[id] = s
	suspend(s)
	return id
end

function socket.start(id, func)
	rsknet_socket_start(id)
	return connect(id, func)
end

return socket