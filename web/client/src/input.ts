import * as Connection from "./connection"
import * as Globals from "./globals"
import * as Types from "./types"

const commandState = new Map<Types.Command, boolean>()

export function setup () {
    document.addEventListener("keydown", (event) => {
        const command = codeToEvent(event.code)
        if (command) {
            sendCommand(command, true)
        }
    })
    document.addEventListener("keyup", (event) => {
        const command = codeToEvent(event.code)
        if (command) {
            sendCommand(command, false)
        }
    })
}

function codeToEvent (code: string): Types.Command | null {
    switch (code) {
    case "KeyW":
    case "ArrowUp":
        return Types.Command.MoveUp
    case "KeyS":
    case "ArrowDown":
        return Types.Command.MoveDown
    default:
        return null
    }
}

let msgId = 0
function sendCommand (command: Types.Command, active: boolean): void {
    if (commandState.get(command) !== active) {
        commandState.set(command, active)
        const msg: Types.IClientMessage = {
            active,
            command,
            id: msgId++,
        }
        Globals.unconfirmedInputs.push(msg)
        Connection.send(msg)
    }
}
