'use strict'

let io = null
let reconnectingText = null
let lastMessage = null
const MIN_WAIT = 100
let wait = MIN_WAIT

function resetWait() {
    wait = MIN_WAIT
}

function incrementWait() {
    const MAX_WAIT = 2000
    if (wait < MAX_WAIT)
        wait *= 1.25
    else
        wait = MAX_WAIT
}

function connect(address) {
    io = new WebSocket(address)
    io.onopen = () => {
        resetWait()
        if (reconnectingText) {
            app.stage.removeChild(reconnectingText)
            reconnectingText = null
        }
    };

    io.onmessage = (msg) => {
        states.push(JSON.parse(msg.data))
    }

    io.onclose = () => {
        if (!reconnectingText && app) {
            reconnectingText = new PIXI.Text('Reconnecting...')
            reconnectingText.anchor.set(0.5)
            reconnectingText.y = 500
            reconnectingText.x = 400
            app.stage.addChild(reconnectingText)
        }
        io = null
        setTimeout(() => {
            incrementWait()
            connect(address);
        }, wait)
    };

    io.onerror = (err) => {
        console.error('Socket encountered error: ', err.message, 'Closing socket')
        io.close()
        io = null
    };
}



function send(data) {
    if (io && io.readyState === 1) {
        console.log('sending data: ', data)
        io.send(data)
    }
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

const addr = window.location.hostname === 'localhost' ? 'ws://localhost:8081' : 'wss://beta.jnferner.com/socket'
connect(addr)
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

let states = []

function play(delta) {
    render(states)
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

const INTERPOLATION_DELTA = 100

function render(states) {
    const now = timestamp()
    const renderTime = now - INTERPOLATION_DELTA
    const index = getIndexOfRenderState(states, renderTime)
    if (index === null)
        return
    states.splice(0, index)
    let interpolatedState = getInterpolatedState(states[0], states[1], renderTime)
    setWorld(interpolatedState)
}

function getIndexOfRenderState(states, renderTime) {
    let found = null
    for (let i = 0; i < states.length; i++) {
        if (states[i].timestamp >= renderTime) {
            if (i !== 0)
                found = i - 1
            break
        }
    }
    return found
}

function getInterpolatedState(from, to, renderTime) {
    const total = to.timestamp - from.timestamp
    const progress = renderTime - from.timestamp
    if (total === 0 || progress === 0)
        return from
    const fraction = progress / total
    let state = from
    state.vel.x += (to.vel.x - from.vel.x) * fraction
    state.vel.y += (to.vel.y - from.vel.y) * fraction
    state.pos.x += (to.pos.x - from.pos.x) * fraction
    state.pos.y += (to.pos.y - from.pos.y) * fraction
    state.timestamp = renderTime
    return state
}

function timestamp() {
    return new Date().getTime()
}

function setWorld(state) {
    for (let blob of blobs) {
        blob.x = state.pos.x
        blob.y = state.pos.y
    }
}