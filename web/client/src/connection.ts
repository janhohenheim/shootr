import * as Display from "./display"
import * as Globals from "./globals"
import * as Types from "./types"

let io: WebSocket | null
const MIN_WAIT = 100
let wait = MIN_WAIT
export function connect (address: string): void {
    io = new WebSocket(address)
    io.onopen = () => {
        resetWait()
        Globals.connectionInfo.visible = false
    }

    io.onmessage = (serializedMsg) => {
        const msg: Types.IServerMessage = JSON.parse(serializedMsg.data, (_, value) => value === "" ? 0 : value)

        switch (msg.opcode) {
        case Types.OpCode.Greeting:
            Globals.setOwnId(msg.payload[0])
            const presentActors = msg.payload[1]
            for (const actor of presentActors) {
                Display.spawnActor(actor)
            }
            break
        case Types.OpCode.Spawn:
            Display.spawnActor(msg.payload)
            break
        case Types.OpCode.Despawn:
            Display.removeActor(msg.payload)
            break
        case Types.OpCode.WorldUpdate:
            const state: Types.IState = {
                actors: msg.payload.actors,
                tick: msg.tick,
                timestamp: performance.now(),
            }
            Globals.states.push(state)
            const index = Globals.unconfirmedInputs.findIndex((input) => input.id === msg.payload.last_input) + 1
            if (index > 0) {
                Globals.unconfirmedInputs.splice(0, index)
            }
            break
        default:
            throw new Error(`Received invalid opcode: ${msg.opcode}`)
        }
    }

    io.onclose = () => {
        Globals.connectionInfo.text = "Attempting to reconnect"
        Globals.connectionInfo.visible = true
        io = null
        setTimeout(() => {
            incrementWait()
            connect(address)
        }, wait)
    }

    io.onerror = () => {
        Globals.connectionInfo.text = "Lost connection to server"
        Globals.connectionInfo.visible = true
        for (const [id, _] of Globals.actors) {
            Display.removeActor(id)
        }
        if (io) {
            io.close()
            io = null
        }
    }
}

export function send (data: Types.IClientMessage): void {
    if (io && io.readyState === 1) {
        io.send(JSON.stringify(data))
    }
}

function resetWait (): void {
    wait = MIN_WAIT
}

function incrementWait (): void {
    const MAX_WAIT = 2000
    if (wait < MAX_WAIT) {
        wait *= 1.25
    } else {
        wait = MAX_WAIT
    }
}
