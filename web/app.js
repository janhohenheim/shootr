'use strict'

const express = require('express')
const dotenv = require('dotenv')

const result = dotenv.config()
if (result.error)
    dotenv.config({
        path: '../.env'
    })

const app = express()
const port = process.env.SITE_PORT

app.use(express.static('public'))

app.get('/', (req, res) => {
    res.sendFile('index.html')
})

app.listen(port, () => {
    console.log('listening on port', port)
})