'use strict'

let io = null

let connectionInfo

const MIN_WAIT = 100
let wait = MIN_WAIT

let states = []

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
        if (connectionInfo) {
            connectionInfo.visible = false
        }
    };

    io.onmessage = (msg) => {
        states.push(JSON.parse(msg.data, (key, value) => value === "" ? 0 : value))
    }

    io.onclose = () => {
        if (connectionInfo) {
            connectionInfo.text = 'Reconnecting...'
            connectionInfo.visible = true
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
        io.send(JSON.stringify(data))
    }
}

const Application = PIXI.Application,
    loader = PIXI.loader,
    resources = PIXI.loader.resources,
    Sprite = PIXI.Sprite

const app = new Application(
    1000, 1000, {
        backgroundColor: 0xFFFFFF,
        antialias: true,
    },
)

const addr = window.location.hostname === 'localhost' ? 'ws://localhost:8081' : 'wss://beta.jnferner.com/socket'
connect(addr)
document.body.appendChild(app.view)

loader
    .add([{
        name: 'pong',
        url: 'assets/pong.json'
    }, ])
    .on('progress', loadProgressHandler)
    .load(setup)

let validKeys = ["ArrowUp", "ArrowDown"]
let keyPressed = {
    ArrowLeft: false,
    ArrowRight: false
}
document.addEventListener("keydown", (event) => {
    sendKeyWithVal(event.key, true)
})
document.addEventListener("keyup", (event) => {
    sendKeyWithVal(event.key, false)
})

function sendKeyWithVal(key, val) {
    if (validKeys.indexOf(key) > -1) {
        sendIfNew(key, val)
    }
}

let msgId = 0

function sendIfNew(key, val) {
    if (keyPressed[key] !== val) {
        keyPressed[key] = val
        let msg = {
            id: msgId,
            key: key,
            pressed: val
        }
        msgId++
        send(msg)
    }
}

function loadProgressHandler(loader, resource) {
    console.log('loading: ' + resource.name + ' (' + resource.url + ')')
    console.log('progress: ' + loader.progress + '%')
    if (resource.error)
        console.error(resource.error)
}

let ball
let player

function setup() {
    const background = new Sprite(resources.pong.textures['fancy-court.png'])
    background.width = 1000
    background.height = 1000
    app.stage.addChild(background)

    connectionInfo = new PIXI.Text('')
    connectionInfo.style.fill = 0xe3e3ed
    connectionInfo.style.dropShadow = true
    connectionInfo.style.dropShadowAlpha = 0.7
    connectionInfo.y = 30
    connectionInfo.x = 40
    app.stage.addChild(connectionInfo)

    ball = new Sprite(resources.pong.textures['fancy-ball.png'])
    ball.anchor.set(0.5)
    app.stage.addChild(ball)

    player = new Sprite(resources.pong.textures['fancy-paddle-green.png'])
    player.anchor.set(0.5)
    app.stage.addChild(player)


    app.ticker.add(gameLoop)
}

let state = connecting

function gameLoop(delta) {
    state()
}

function connecting() {
    connectionInfo.text = 'Connecting...'
    connectionInfo.visible = true

    const renderTime = getRenderTime()
    const index = getIndexOfRenderState(states, renderTime)
    if (index >= 0) {
        connectionInfo.visible = false
        state = play
    }
}

function play() {
    render(states)
}


function render(states) {
    const renderTime = getRenderTime()
    const index = getIndexOfRenderState(states, renderTime)
    if (index < 0)
        return
    states.splice(0, index)
    let interpolatedState = getInterpolatedState(states[0], states[1], renderTime)
    setWorld(interpolatedState)
}

function getRenderTime() {
    const now = new Date().getTime()
    const INTERPOLATION_DELTA = 200
    return now - INTERPOLATION_DELTA
}

function getIndexOfRenderState(states, renderTime) {
    const found = states.findIndex((state) => state.timestamp >= renderTime)
    return found - 1
}

function getInterpolatedState(from, to, renderTime) {
    const total = to.timestamp - from.timestamp
    const progress = renderTime - from.timestamp
    if (total === 0 || progress === 0)
        return from
    const fraction = progress / total
    let state = from

    state.ball.vel.x += (to.ball.vel.x - from.ball.vel.x) * fraction
    state.ball.vel.y += (to.ball.vel.y - from.ball.vel.y) * fraction
    state.ball.pos.x += (to.ball.pos.x - from.ball.pos.x) * fraction
    state.ball.pos.y += (to.ball.pos.y - from.ball.pos.y) * fraction

    state.player.acc.x += (to.player.acc.x - from.player.acc.x) * fraction
    state.player.acc.y += (to.player.acc.y - from.player.acc.y) * fraction
    state.player.vel.x += (to.player.vel.x - from.player.vel.x) * fraction
    state.player.vel.y += (to.player.vel.y - from.player.vel.y) * fraction
    state.player.pos.x += (to.player.pos.x - from.player.pos.x) * fraction
    state.player.pos.y += (to.player.pos.y - from.player.pos.y) * fraction

    state.timestamp = renderTime
    return state
}


function setWorld(state) {
    ball.x = state.ball.pos.x
    ball.y = state.ball.pos.y

    player.x = state.player.pos.x
    player.y = state.player.pos.y

    const maxVel = Math.max(state.ball.vel.x, state.ball.vel.y)
    const strength = Math.pow(Math.atan(Math.pow((maxVel / 10), 1.5)), 2) - 0.2
    if (strength > 0) {
        const blurFilter = new PIXI.filters.BlurFilter(strength, 1, 1)
        ball.filters = [blurFilter]
    }
}