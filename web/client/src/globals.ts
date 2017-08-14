import * as Types from "./types"

export const actors = new Map<Types.Id, PIXI.Sprite>()
export let ownId: Types.Id
export function setOwnId (id: Types.Id): void {
    ownId = id
}
export let connectionInfo: PIXI.Text
export function setConnectionInfo (sprite: PIXI.Text): void {
    connectionInfo = sprite
}
export const states: Types.IState[] = []
export const unconfirmedInputs: Types.IClientMessage[] = []
