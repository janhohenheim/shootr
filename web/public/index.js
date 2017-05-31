'use strict'
const io = new WebSocket('ws://localhost:8081', 'rust-websocket')
io.onmessage = (msg) => {
    console.log('Received message: ', msg)
}

const Application = PIXI.Application,
    loader = PIXI.loader,
    resources = PIXI.loader.resources,
    Sprite = PIXI.Sprite

const app = new Application(
    800, 600,
    {
        backgroundColor: 0x1099bb,
        autoResize: true,
        antialias: true,
        resolution: 1,
    },
)
document.body.appendChild(app.view)

loader
    .add([
        {name: 'dungeonAtlas', url: 'assets/dungeon.json'},
    ])
    .on('progress', loadProgressHandler)
    .load(setup)

app.ticker.add(gameLoop)

function loadProgressHandler(loader, resource) {
    console.log('loading: ' + resource.name + ' (' + resource.url + ')')
    console.log('progress: ' + loader.progress + '%')
    if (resource.error)
        console.error(resource.error)
}

let blobs = []
let explorer = null
function setup() {
    explorer = new Sprite(resources.dungeonAtlas.textures['explorer.png'])
    explorer.y = app.view.height / 2 - explorer.height / 2

    app.stage.addChild(explorer)
    for (let i = 0; i < 10; i++) {
        const blob = new Sprite(resources.dungeonAtlas.textures['blob.png'])
        blob.x = rand(0, app.view.width)
        blob.y = rand(0, app.view.height)
        blob.vx = rand(-2.0, 2.0)
        blob.vy = rand(-2.0, 2.0)
        blob.vrotation = rand(0.05, 0.15)

        app.stage.addChild(blob)
        blobs.push(blob)
    }

    for (let obj of app.stage.children)
        initObj(obj)
}

function rand(min, max) {
    return Math.floor(Math.random() * (max - min + 1)) + min
}

let state = play

function gameLoop(delta) {
    state(delta)
}

function play(delta) {
    for (let blob of blobs) {
        if (blob.x <= 0 || blob.x >= app.view.width)
            blob.vx = -blob.vx
        if (blob.y <= 0 || blob.y >= app.view.height)
            blob.vy = -blob.vy
    }
    for (let obj of app.stage.children)
        move(obj, delta)
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

function move(obj, delta) {
    obj.x += obj.vx * delta
    if (obj.x > app.view.width)
        obj.x = app.view.width
    else if (obj.x < 0)
        obj.x = 0

    obj.y += obj.vy * delta
    if (obj.y > app.view.height)
        obj.y = app.view.height
    else if (obj.y < 0)
        obj.y = 0

    obj.rotation += obj.vrotation * delta
}


const left = keyboard('ArrowLeft', 'a'),
    up = keyboard('ArrowUp', 'd'),
    right = keyboard('ArrowRight', 'h'),
    down = keyboard('ArrowDown', 's')

left.press = () => {
    explorer.vx = -5
}

left.release = () => {
    if (!right.isDown)
        explorer.vx = 0
}

up.press = () => {
    explorer.vy = -5
}
up.release = () => {
    if (!down.isDown)
        explorer.vy = 0
}

right.press = () => {
    explorer.vx = 5
}
right.release = () => {
    if (!left.isDown)
        explorer.vx = 0
}

down.press = () => {
    explorer.vy = 5
}
down.release = () => {
    if (!up.isDown)
        explorer.vy = 0
}


function keyboard() {
    const keyHandler = {}
    keyHandler.key = arguments
    keyHandler.isDown = false
    keyHandler.isUp = true
    keyHandler.press = undefined
    keyHandler.release = undefined

    keyHandler.downHandler = event => {
        for (let key of keyHandler.key)
            if (event.key === key) {
                if (keyHandler.isUp && keyHandler.press)
                    keyHandler.press()
                keyHandler.isDown = true
                keyHandler.isUp = false
                event.preventDefault()
            }
    }

    keyHandler.upHandler = event => {
        for (let key of keyHandler.key)
            if (event.key === key) {
                if (keyHandler.isDown && keyHandler.release)
                    keyHandler.release()
                keyHandler.isDown = false
                keyHandler.isUp = true
                event.preventDefault()
            }
    }

    window.addEventListener(
        'keydown', keyHandler.downHandler.bind(keyHandler),
    )
    window.addEventListener(
        'keyup', keyHandler.upHandler.bind(keyHandler),
    )
    return keyHandler
}
