'use strict'
const io = new WebSocket('ws://localhost:8081', 'rust-websocket')
io.onmessage = (msg) => {
    console.log('Received message: ', msg)
}