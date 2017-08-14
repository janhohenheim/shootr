export interface IState {
    actors: IActor[], // TODO: Somehow use a Map for this
    timestamp: number,
    tick: number,
}

export type Id = string
export interface IActor {
    id: Id,
    kind: ActorKind
}
export enum ActorKind {
    Player = "Player",
    Ball = "Ball",
}

export enum OpCode {
    Greeting = "Greeting",
    Spawn = "Spawn",
    Despawn = "Despawn",
    WorldUpdate = "WorldUpdate",
}

export enum Command {
    MoveUp = "MoveUp",
    MoveDown = "MoveDown",
}

export interface IClientMessage {
    active: boolean,
    command: Command,
    id: number
}

export interface IServerMessage {
    opcode: OpCode,
    payload: any,
    tick: number
}

export interface IVector {
    x: number,
    y: number,
}
