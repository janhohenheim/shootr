import * as Globals from "./globals"
import * as Types from "./types"

export function getRenderTime (): number {
    const lerpRatio = 2
    const updateRate = 30
    const delay = Math.floor(lerpRatio * 1000 / updateRate)
    const now = Math.floor(performance.now())
    return now - delay
}

export function getIndexOfRenderState (renderTime: number): number {
    const found = Globals.states.findIndex((state) => state.timestamp >= renderTime)
    return found - 1
}

export function getInterpolatedState (from: Types.IState, to: Types.IState, renderTime: number): Types.IState {
    const total = to.timestamp - from.timestamp
    const progress = renderTime - from.timestamp
    if (total === 0 || progress === 0) { return from }
    const fraction = progress / total
    const state: Types.IState = JSON.parse(JSON.stringify(from))
    for (const id of Object.keys(state.actors)) {
        const actor = state.actors[id]
        const toActor = to.actors[id]
        if (!toActor) { continue }
        const fromActor = from.actors[id]

        actor.pos.x += (toActor.pos.x - fromActor.pos.x) * fraction
        actor.pos.y += (toActor.pos.y - fromActor.pos.y) * fraction
        actor.vel.x += (toActor.vel.x - fromActor.vel.x) * fraction
        actor.vel.y += (toActor.vel.y - fromActor.vel.y) * fraction
    }
    state.timestamp = renderTime
    return state
}
