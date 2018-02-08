const cbor = require('cbor')
const chalk = require('chalk')
const crypto = require('crypto')
const dgram = require('dgram')
const pkg = require('./package.json')
const sodium = require('sodium-native')

const PORT = +(process.env.PORT || 6776)
const CAST = '224.0.247.51'
const ID = crypto.randomBytes(16).toString('base64').replace(/[^\w]+/g, '')

const config = require('./constellationd.json')
const KEY = config.key
const SECRET = Buffer.from(config.secret)

const client = dgram.createSocket({ type: 'udp4', reuseAddr: true })

client.on('listening', () => {
  const address = client.address()
  console.log('UDP Client listening on ' + address.address + ":" + address.port)
  client.setBroadcast(true)
  client.setMulticastLoopback(true)
  client.setMulticastTTL(128)
  client.addMembership(CAST)
  message('Hello')
  setInterval(() => {
      message('Ping')
  }, 10000)
})

client.on('message', (message, remote) => {
    const msg = cbor.decode(message)
    try {
        var { v, key, nonce, body } = msg
        if (v !== 0) throw new Error('Wrong version')
        if (key !== KEY) throw new Error('Wrong key')
    } catch (err) {
        return console.log('←', message)
    }

    if (body.length > sodium.crypto_secretbox_MACBYTES) {
        const plain = Buffer.alloc(body.length - sodium.crypto_secretbox_MACBYTES)
        if (sodium.crypto_secretbox_open_easy(plain, body, nonce, SECRET)) {
            body = cbor.decode(plain)
        }
    }

    console.log(ts(), chalk.bold.blueBright(' <- '), body)
})

client.bind(PORT)

function message (kind, args = {}) {
    const data = {
        agent: [pkg.name, pkg.version],
        id: ID,
        kind,
        ...args
    }

    const message = cbor.encode(data)
    const msg = Buffer.from(message)

    const nonce = crypto.randomBytes(sodium.crypto_secretbox_NONCEBYTES)
    const cipher = Buffer.alloc(msg.length + sodium.crypto_secretbox_MACBYTES)
    sodium.crypto_secretbox_easy(cipher, msg, nonce, SECRET)

    const envelope = Buffer.from(cbor.encode({
        v: 0,
        key: KEY,
        nonce,
        body: cipher,
    }))

    client.send(envelope, 0, envelope.length, PORT, CAST)
    console.log(ts(), chalk.bold.magentaBright(' -> '), data)
}

function ts () {
    const now = new Date
    return chalk.grey(
        `${now}`.split(' ')[4] + '.' + now.getMilliseconds()
    )
}
