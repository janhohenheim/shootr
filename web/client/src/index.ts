"use strict"

import * as Display from "./display"
import * as Globals from "./globals"
import * as Input from "./input"
import * as Interpolation from "./interpolation"
import * as Types from "./types"

Display.setup(gameLoop)
Input.setup()

let onGameUpdate = connecting
function gameLoop (delta: number): void {
    onGameUpdate(delta)
}
function connecting (delta: number): void {
    const txt = "Connecting..."
    if (Globals.connectionInfo.text !== txt) {
        Globals.connectionInfo.text = txt
        Globals.connectionInfo.visible = true
    }

    const renderTime = Interpolation.getRenderTime()
    const index = Interpolation.getIndexOfRenderState(renderTime)
    if (index >= 0) {
        Globals.connectionInfo.visible = false
        onGameUpdate = play
    }
}

function render (): void {
    const renderTime = Interpolation.getRenderTime()
    const index = Interpolation.getIndexOfRenderState(renderTime)
    if (index < 0) {
        console.log("Waiting for more recent snapshot")
        return
    }
    Globals.states.splice(0, index)
    const interpolatedState = Interpolation.getInterpolatedState(Globals.states[0], Globals.states[1], renderTime)
    setWorld(interpolatedState)
}

function play (): void {
    render()
}

function setWorld (state: Types.IState): void {
    for (const id of Object.keys(state.actors)) {
        const liveActor = Globals.actors.get(id)
        const stateActor = state.actors[id]
        if (!liveActor || !stateActor) {
            continue
        }
        liveActor.x = stateActor.pos.x
        liveActor.y = stateActor.pos.y
        if (stateActor.vel) {
            Display.setBlur(liveActor, stateActor.vel)
        }
    }
}
