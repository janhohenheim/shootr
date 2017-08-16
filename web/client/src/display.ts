import * as Pixi from "pixi.js"
import * as Connection from "./connection"
import * as Globals from "./globals"
import * as Types from "./types"

export const GAME_WIDTH = 1000
export const GAME_HEIGHT = 1000
export const app = new Pixi.Application(
    screen.availWidth, screen.availHeight, {
        antialias: true,
        backgroundColor: 0xFFFFFF,
    },
)

let onGameUpdate: (delta: number) => void
export function setup (gameLoop: (delta: number) => void): void {
    onGameUpdate = gameLoop
    document.body.appendChild(app.view)
    Pixi.loader
        .add([{
            name: "pong",
            url: "assets/pong.json",
        } ])
        .on("progress", loadProgressHandler)
        .load(pixiSetup)
}

const resources = Pixi.loader.resources
export function spawnActor (actor: Types.IActor): void {
    let texture: string
    let height: number
    let width: number
    switch (actor.kind) {
    case Types.ActorKind.Player:
        texture = "fancy-paddle-green.png"
        height = 75
        width = 15
        break
    case Types.ActorKind.Ball:
        texture = "fancy-ball.png"
        height = 15
        width = 15
        break
    default:
        throw new Error(`Tried to spawn invalid kind of actor: ${actor.kind}`)
    }

    if (!(resources.pong && resources.pong.textures)) {
        throw new Error("Failed to spawn actor: Pixi was not initialized properly")
    }

    const sprite = new Pixi.Sprite(resources.pong.textures[texture])
    sprite.anchor.set(0.5)
    sprite.width = width
    sprite.height = height
    app.stage.addChild(sprite)
    Globals.actors.set(actor.id, sprite)
}

export function removeActor (id: Types.Id): void {
    const actor = Globals.actors.get(id)
    if (!actor) {
        throw new Error(`Failed to remove actor: No actor found with id ${id}`)
    }
    app.stage.removeChild(actor)
    Globals.actors.delete(id)
}

export function setBlur (obj: Pixi.Sprite, vel: Types.IVector): void {
    const maxVel = Math.max(Math.abs(vel.x), Math.abs(vel.y))
    const strength = Math.pow(Math.atan(Math.pow((maxVel / 10), 1.5)), 2) - 0.2
    if (strength > 0.5) {
        const blurFilter = new Pixi.filters.BlurFilter(strength, 1, 1)
        obj.filters = [blurFilter]
    } else {
        obj.filters = []
    }
}

function pixiSetup (): void {
    if (!(resources.pong && resources.pong.textures)) {
        throw new Error("Failed to setup stage: Pixi was not initialized properly")
    }
    const background = new Pixi.Sprite(resources.pong.textures["fancy-court.png"])
    background.width = GAME_WIDTH
    background.height = GAME_HEIGHT
    app.stage.addChild(background)

    Globals.setConnectionInfo(new Pixi.Text(""))
    Globals.connectionInfo.style.fill = 0xe3e3ed
    Globals.connectionInfo.style.dropShadow = true
    Globals.connectionInfo.style.dropShadowAlpha = 0.7
    Globals.connectionInfo.y = 30
    Globals.connectionInfo.x = GAME_WIDTH - 300
    app.stage.addChild(Globals.connectionInfo)

    resize()
    window.addEventListener("resize", resize)

    const addr = window.location.hostname === "localhost" ? "ws://localhost:8081" : "wss://beta.jnferner.com/socket"
    Connection.connect(addr)
    app.ticker.add(onGameUpdate)
}

function resize (): void {
    let ratio = window.innerWidth / GAME_WIDTH
    if (GAME_HEIGHT * ratio > window.innerHeight) { ratio = window.innerHeight / GAME_HEIGHT }
    // TODO: types say that width and height don't exist (workaround with square brackets used)
    const width = app.stage.width = GAME_WIDTH * ratio
    const height = app.stage.height = GAME_HEIGHT * ratio
    app.renderer.resize(width, height)
}

function loadProgressHandler (loader: Pixi.loaders.Loader, resource: Pixi.loaders.Resource): void {
    console.log("loading: " + resource.name + " (" + resource.url + ")")
    console.log("progress: " + loader.progress + "%")
    if (resource.error) {
        console.error(resource.error)
    }
}
