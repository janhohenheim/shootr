'use strict'

let io = null
let reconnectingID = null

function start_websocket(wsLocation) {
    io = new WebSocket(wsLocation)
    io.onopen = (event) => {
        if (!reconnectingID)
            return;
        window.clearInterval(reconnectingID)
        reconnectingID = 0
        if (reconnectingText) {
            app.stage.removeChild(reconnectingText)
            reconnectingText = null
        }
    }

    io.onmessage = (msg) => {
        lastMessage = msg.data
    }

    io.onclose = () => {
        if (reconnectingID)
            return;
        reconnectingText = new PIXI.Text('Reconnecting...')
        reconnectingText.anchor.set(0.5)
        reconnectingText.x = 400
        reconnectingText.y = 500

        app.stage.addChild(reconnectingText)

        io = null
        reconnectingID = setInterval(() => {
            start_websocket(wsLocation)
        }, 1000)
    }
}


let lastMessage = undefined
let reconnectingText = undefined;


function send(data) {
    if (!io || !reconnectingID)
        return;
    console.log('sending data: ', data)
    io.send(data)
}

const Application = PIXI.Application,
    loader = PIXI.loader,
    resources = PIXI.loader.resources,
    Sprite = PIXI.Sprite

const app = new Application(
    1000, 1000, {
        backgroundColor: 0x1099bb,
        autoResize: true,
        antialias: true,
        resolution: 1,
    },
)
start_websocket('wss://localhost:8081')
document.body.appendChild(app.view)

loader
    .add([{
        name: 'dungeonAtlas',
        url: 'assets/dungeon.json'
    }, ])
    .on('progress', loadProgressHandler)
    .load(setup)

app.ticker.add(gameLoop)

document.addEventListener("keydown", (event) => {
    send('keydown:' + event.key)
})
document.addEventListener("keyup", (event) => {
    send('keyup:' + event.key)
})
document.addEventListener("mousemove", (event) => {
    //send('mousemove:' + event.clientX + ' ' + event.clientY)
})

function loadProgressHandler(loader, resource) {
    console.log('loading: ' + resource.name + ' (' + resource.url + ')')
    console.log('progress: ' + loader.progress + '%')
    if (resource.error)
        console.error(resource.error)
}

let blobs = []

function setup() {
    for (let i = 0; i < 1; i++) {
        const blob = new Sprite(resources.dungeonAtlas.textures['blob.png'])
        app.stage.addChild(blob)
        blobs.push(blob)
    }

    for (let obj of app.stage.children)
        initObj(obj)
}

let state = play

function gameLoop(delta) {
    state(delta)
}

function play(delta) {
    if (!lastMessage)
        return
    const update = JSON.parse(lastMessage)
    for (let blob of blobs) {
        blob.x = update.pos.x
        blob.y = update.pos.y
    }
}

function initObj(obj) {
    if (obj.vx === undefined)
        obj.vx = 0
    if (obj.vy === undefined)
        obj.vy = 0
    if (obj.vrotation === undefined)
        obj.vrotation = 0
    obj.anchor.set(0.5)
}