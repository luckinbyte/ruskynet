local rsknet = require "../lualib/rsknet"
local socket = require "../lualib/socket"

print("load socket success")

rsknet.start(function()
    print("user server start!!")
	--rsknet.exit()
    local listen_socket = socket.listen("0.0.0.0", 4276)
    socket.start(listen_socket, function(id, addr)
        print("listen_socket get client_fd", id, type(id))
		socket.register_recieve(id, function(...)
            print("receive data",...)
            socket.write(id, "hello_from_main_lua")
        end)
		--socket.start(id)
		--skynet.fork(console_main_loop, id , print, addr)
	end)
end)
