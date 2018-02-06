const crypto = require('crypto')
const dgram = require('dgram')
const msgpack = require('msgpack5')()
const pkg = require('./package.json')
const sodium = require('sodium-native')

const PORT = +(process.env.PORT || 6776)
const CAST = '224.0.247.51'
const ID = crypto.randomBytes(16).toString('base64').replace(/[^\w]+/g, '')

const KEY = 'DopdJoNKELA9bwxaXibc1w'
const SECRET = Buffer.from([119, 17, 247, 68, 67, 146, 203, 92, 62, 134, 39, 34, 240, 64, 131, 125, 218, 235, 91, 119, 157, 225, 13, 248, 10, 119, 164, 125, 211, 137, 191, 88])

const client = dgram.createSocket({ type: 'udp4', reuseAddr: true })

client.on('listening', () => {
  const address = client.address()
  console.log('UDP Client listening on ' + address.address + ":" + address.port)
  client.setBroadcast(true)
  client.setMulticastLoopback(true)
  client.setMulticastTTL(128)
  client.addMembership(CAST)
  message('Hello')
  setInterval(() => message('Ping'), 5000)
})

client.on('message', (message, remote) => {
    const msg = msgpack.decode(message)
    try {
        try {
            var [v, key, nonce, body] = msg
        } catch (e) {
            var { v, key, nonce, body } = msg
            if (v !== 0) throw new Error('Wrong version')
            if (key !== KEY) throw new Error('Wrong key')
        }
    } catch (err) {
        return console.log('←', message)
    }

    body = Buffer.from(body)
    const plain = Buffer.alloc(body.length - sodium.crypto_secretbox_MACBYTES)
    if (sodium.crypto_secretbox_open_easy(plain, body, nonce, SECRET)) {
        body = JSON.parse(plain)
    }

    console.log('←', { v, key, nonce, body })
})

client.bind(PORT)

function message (body) {
    const message = JSON.stringify({
        agent: [pkg.name, pkg.version],
        id: ID,
        kind: body
    })
    const msg = Buffer.from(message)

    const nonce = crypto.randomBytes(sodium.crypto_secretbox_NONCEBYTES)
    const cipher = Buffer.alloc(msg.length + sodium.crypto_secretbox_MACBYTES)
    sodium.crypto_secretbox_easy(cipher, msg, nonce, SECRET)

    const envelope = Buffer.from(msgpack.encode({
        v: 0,
        key: KEY,
        nonce,
        body: Array.from(cipher.values()),
    }))

    client.send(envelope, 0, envelope.length, PORT, CAST)
    console.log('→', message)
}
