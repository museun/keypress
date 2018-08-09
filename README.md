# keypress
sends keypresses for a specific window over a websocket. what could go wrong.

this listens for a keypresses for a specific window and sends them out to a websocket
a simple html file is included that can be used with obs

this is obviously windows only

keypressed.toml is where the configuration is, it has 3 keys you can set, and the address 
for the websocket server to listen on.

exit_key is the key that'll instantly kill the program
watch_key is the key that'll have it watch the window that has focused
pause-key is the key that'll pause all event processing until toggled
host is in an address:port format
