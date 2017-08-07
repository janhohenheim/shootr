'use strict'

let io: WebSocket | null
let connectionInfo: PIXI.Text
let pingInfo: PIXI.Text
let states: IState[] = []

interface IState {
    actors: any, // TODO: Add type
    timestamp: number,
}

interface IInput {
    id: number,
    key: string,
    pressed: boolean,
}

enum OpCode {
    Greeting = 'Greeting',
    Spawn = 'Spawn',
    Despawn = 'Despawn',
    WorldUpdate = 'WorldUpdate',
    Ping = 'Ping',
}

enum ActorKind {
    Player = 'Player',
    Ball = 'Ball',
}

function setPingInfo (ping) {
    pingInfo.text = 'Ping: ' + ping
    let fill: number
    if (ping < 100) {
        fill = 0x57fc20
    } else if (ping < 200) {
        fill = 0xfc9520
    } else {
        fill = 0xef1c2a
    }
    pingInfo.style.fill = fill
}

let ownId: number
let localClockOffset: number

interface IServerMessage {
    opcode: OpCode,
    payload: any,
    server_time: number
}

const unconfirmedInputs: IInput[] = []
const MIN_WAIT = 100
let wait = MIN_WAIT
let actors = {}
function connect (address) {
    io = new WebSocket(address)
    io.onopen = () => {
        resetWait()
        connectionInfo.visible = false
    }

    io.onmessage = (serializedMsg) => {
        const msg: IServerMessage = JSON.parse(serializedMsg.data, (_, value) => value === '' ? 0 : value)

        switch (msg.opcode) {
        case OpCode.Greeting:
            ownId = msg.payload[0]
            const presentActors = msg.payload[1]
            localClockOffset = performance.now() - msg.server_time

            for (const actor of presentActors) {
                spawnActor(actor)
            }
            break
        case OpCode.Spawn:
            spawnActor(msg.payload)
            break
        case OpCode.Despawn:
            removeActor(msg.payload)
            break
        case OpCode.WorldUpdate:
            const state = {
                actors: msg.payload.actors,
                timestamp: msg.server_time + localClockOffset,
            }
            states.push(state)
            const index = unconfirmedInputs.findIndex((input) => input.id === msg.payload.last_input) + 1
            if (index > 0) { unconfirmedInputs.splice(0, index) }
            break
        case OpCode.Ping:
            const pong = {
                id: msg.payload,
            }
            send(pong)
            break
        default:
            throw new Error(`Received invalid opcode: ${msg.opcode}`)
        }
    }

    io.onclose = () => {
        connectionInfo.text = 'Attempting to reconnect'
        connectionInfo.visible = true
        io = null
        setTimeout(() => {
            incrementWait()
            connect(address)
        }, wait)
    }

    io.onerror = () => {
        connectionInfo.text = 'Lost connection to server'
        connectionInfo.visible = true
        for (const id of Object.keys(actors)) { removeActor(id) }
        if (io) {
            io.close()
            io = null
        }
    }
}

function resetWait () {
    wait = MIN_WAIT
}

function incrementWait () {
    const MAX_WAIT = 2000
    if (wait < MAX_WAIT) { wait *= 1.25 } else { wait = MAX_WAIT }
}

function send (data) {
    if (io && io.readyState === 1) { io.send(JSON.stringify(data)) }
}

const GAME_WIDTH = 1000
const GAME_HEIGHT = 1000
const app = new PIXI.Application(
    screen.availWidth, screen.availHeight, {
        antialias: true,
        backgroundColor: 0xFFFFFF,
    },
)

document.body.appendChild(app.view)

PIXI.loader
    .add([{
        name: 'pong',
        url: 'assets/pong.json',
    } ])
    .on('progress', loadProgressHandler)
    .load(setup)

let validKeys = ['ArrowUp', 'ArrowDown']
let keyPressed = {
    ArrowLeft: false,
    ArrowRight: false,
}
document.addEventListener('keydown', (event) => {
    sendKeyWithVal(event.key, true)
})
document.addEventListener('keyup', (event) => {
    sendKeyWithVal(event.key, false)
})

function sendKeyWithVal (key, val) {
    if (validKeys.indexOf(key) > -1) {
        sendIfNew(key, val)
    }
}

let msgId = 1
function sendIfNew (key, val) {
    if (keyPressed[key] !== val) {
        keyPressed[key] = val
        const msg = {
            id: msgId,
            key,
            pressed: val,
        }
        msgId++
        unconfirmedInputs.push(msg)
        send(msg)
    }
}

function loadProgressHandler (loader, resource) {
    console.log('loading: ' + resource.name + ' (' + resource.url + ')')
    console.log('progress: ' + loader.progress + '%')
    if (resource.error) { console.error(resource.error) }
}

const resources = PIXI.loader.resources
function setup () {
    if (!(resources.pong && resources.pong.textures)) {
        throw new Error('Failed to setup stage: PIXI was not initialized properly')
    }
    const background = new PIXI.Sprite(resources.pong.textures['fancy-court.png'])
    background.width = GAME_WIDTH
    background.height = GAME_HEIGHT
    app.stage.addChild(background)

    connectionInfo = new PIXI.Text('')
    connectionInfo.style.fill = 0xe3e3ed
    connectionInfo.style.dropShadow = true
    connectionInfo.style.dropShadowAlpha = 0.7
    connectionInfo.y = 30
    connectionInfo.x = GAME_WIDTH - 300
    app.stage.addChild(connectionInfo)

    pingInfo = new PIXI.Text('Ping: Calculating...')
    pingInfo.style.fill = 0xe3e3ed
    pingInfo.style.dropShadow = true
    pingInfo.style.dropShadowAlpha = 0.7
    pingInfo.y = 30
    pingInfo.x = 40
    app.stage.addChild(pingInfo)

    resize()
    window.addEventListener('resize', resize)

    setInterval(() => setPingInfo(getDelay().ping), 1000)
    const addr = window.location.hostname === 'localhost' ? 'ws://localhost:8081' : 'wss://beta.jnferner.com/socket'
    connect(addr)
    app.ticker.add(gameLoop)
}

function getDelay () {
    const noDelay = {
        clock: 0,
        ping: 0,
    }
    if (states.length === 0 || !ownId) {
        return noDelay
    }

    const players = states[states.length - 1].actors
    if (!players[ownId]) {
        return noDelay
    }
    return players[ownId].delay
}

function resize () {
    let ratio = window.innerWidth / GAME_WIDTH
    if (GAME_HEIGHT * ratio > window.innerHeight) { ratio = window.innerHeight / GAME_HEIGHT }
    // TODO: types say that width and height don't exist (workaround with square brackets used)
    const width = app.stage.width = GAME_WIDTH * ratio
    const height = app.stage.height = GAME_HEIGHT * ratio
    app.renderer.resize(width, height)
}

let onGameUpdate = connecting

function gameLoop () {
    onGameUpdate()
}

function connecting () {
    const txt = 'Connecting...'
    if (connectionInfo.text !== txt) {
        connectionInfo.text = txt
        connectionInfo.visible = true
    }

    const renderTime = getRenderTime()
    const index = getIndexOfRenderState(renderTime)
    if (index >= 0) {
        connectionInfo.visible = false
        onGameUpdate = play
    }
}

function play () {
    render()
}

function render () {
    const renderTime = getRenderTime()
    const index = getIndexOfRenderState(renderTime)
    if (index < 0) {
        console.log('Waiting for more recent snapshot')
        return
    }
    states.splice(0, index)
    const interpolatedState = getInterpolatedState(states[0], states[1], renderTime)
    setWorld(interpolatedState)
}

function getRenderTime () {
    const lerpRatio = 2
    const updateRate = 30
    const delay = Math.floor(lerpRatio * 1000 / updateRate)
    const now = Math.floor(performance.now())
    return now - delay
}

function getIndexOfRenderState (renderTime) {
    const found = states.findIndex((state) => state.timestamp >= renderTime)
    return found - 1
}

function getInterpolatedState (from, to, renderTime) {
    const total = to.timestamp - from.timestamp
    const progress = renderTime - from.timestamp
    if (total === 0 || progress === 0) { return from }
    const fraction = progress / total
    const state = JSON.parse(JSON.stringify(from)).actors
    for (const id of Object.keys(state)) {
        const actor = state[id]
        const toActor = to[id]
        if (!toActor) { continue }
        const fromActor = from[id]

        actor.pos.x += (toActor.pos.x - fromActor.pos.x) * fraction
        actor.pos.y += (toActor.pos.y - fromActor.pos.y) * fraction
        actor.vel.x += (toActor.vel.x - fromActor.vel.x) * fraction
        actor.vel.y += (toActor.vel.y - fromActor.vel.y) * fraction
    }
    state.timestamp = renderTime
    return state
}

function setWorld (state) {
    for (const id of Object.keys(state)) {
        const liveActor = actors[id]
        const stateActor = state[id]
        if (!liveActor || !stateActor) { continue }
        liveActor.x = stateActor.pos.x
        liveActor.y = stateActor.pos.y
        if (stateActor.vel) { setBlur(liveActor, stateActor.vel) }
    }
}

function setBlur (obj, vel) {
    const maxVel = Math.max(Math.abs(vel.x), Math.abs(vel.y))
    const strength = Math.pow(Math.atan(Math.pow((maxVel / 10), 1.5)), 2) - 0.2
    if (strength > 0.5) {
        const blurFilter = new PIXI.filters.BlurFilter(strength, 1, 1)
        obj.filters = [blurFilter]
    } else {
        obj.filters = []
    }
}

function spawnActor (actor) {
    let texture: string
    let height: number
    let width: number
    switch (actor.kind) {
    case ActorKind.Player:
        texture = 'fancy-paddle-green.png'
        height = 75
        width = 15
        break
    case ActorKind.Ball:
        texture = 'fancy-ball.png'
        height = 15
        width = 15
        break
    default:
        throw new Error(`Tried to spawn invalid kind of actor: ${actor.kind}`)
    }
    // TODO: i'm sorry for this too
    const sprite = new PIXI.Sprite(resources.pong && resources.pong.textures && resources.pong.textures[texture])
    sprite.anchor.set(0.5)
    sprite.width = width
    sprite.height = height
    app.stage.addChild(sprite)
    actors[actor.id] = sprite
}

function removeActor (id) {
    const actor = actors[id]
    app.stage.removeChild(actor)
    delete actors[id]
}
