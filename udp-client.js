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
})

client.on('message', (message, remote) => {
    try {
        const v = message.readUInt8(0)
        if (v !== 1) throw new Error('Wrong version: ' + v)

        const hlen = message.readUInt8(1)
        if (hlen < 1) throw new Error('Zero-length header')

        const header = message.slice(2, 2 + hlen)
        if (header.length < hlen) throw new Error('Bad header length: ' + header.length)

        var [ key, nonce, plen ] = cbor.decode(header)
        if (key !== KEY) throw new Error('Wrong key: ' + key)

        var body = message.slice(2 + hlen, 2 + hlen + plen)
        if (body.length < plen) throw new Error('Bad payload length: ' + body.length)
    } catch (err) {
        return console.log('←', message)
    }

    if (body.length > sodium.crypto_secretbox_MACBYTES) {
        const plain = Buffer.alloc(body.length - sodium.crypto_secretbox_MACBYTES)
        if (sodium.crypto_secretbox_open_easy(plain, body, nonce, SECRET)) {
            body = cbor.decode(plain)
        }
    }

    const len = message.length
    console.log(ts(),
        chalk.bold.blueBright(' <- '),
        chalk[
            len < 512 ? 'green' :
            (len > 1200 ? 'red' : 'yellow')
        ](`${len} bytes`),
        JSON.stringify(body)
    )
})

client.bind(PORT)

function message (kind, args = {}) {
    const data = {
        agent: ['udp-client', pkg.version],
        id: ID,
        body: [ kind, args ]
    }

    const message = cbor.encode(data)
    const msg = Buffer.from(message)

    const nonce = crypto.randomBytes(sodium.crypto_secretbox_NONCEBYTES)
    const payload = Buffer.alloc(msg.length + sodium.crypto_secretbox_MACBYTES)
    sodium.crypto_secretbox_easy(payload, msg, nonce, SECRET)

    const header = Buffer.from(cbor.encode([
        KEY,
        nonce,
        payload.length,
    ]))

    const envelope = Buffer.alloc(2 + header.length + payload.length)
    envelope.writeUInt8(1, 0)
    envelope.writeUInt8(header.length, 1)
    header.copy(envelope, 2)
    payload.copy(envelope, 2 + header.length)

    client.send(envelope, 0, envelope.length, PORT, CAST)
    console.log(ts(), chalk.bold.magentaBright(' -> '), data)
}

function ts () {
    const now = new Date
    return chalk.grey(
        `${now}`.split(' ')[4] + '.' + now.getMilliseconds()
    )
}

const readline = require('readline')
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
})

rl.on('line', (line) => message('Arbitrary', line))
