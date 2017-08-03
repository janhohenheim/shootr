'use strict'

const express = require('express')
const dotenv = require('dotenv')

let result = dotenv.config()
if (result.error) {
    result = dotenv.config({
        path: '../../.env'
    })
    if (result.error)
        throw result.error
}

const app = express()
const port = readEnvVar('SITE_PORT')

app.use(express.static('../client/public'))

app.get('/', (req, res) => {
    res.sendFile('index.html')
})

app.listen(port, () => {
    console.log('listening on port', port)
})

function readEnvVar(envvar) {
    const val = process.env[envvar]
    if (!val)
        throw envvar + " must be specified. \
Did you forget to add it to your .env file?"
    return val
}