const crypto = require('crypto')
const dgram = require('dgram')
const pkg = require('./package.json')

const PORT = +(process.env.PORT || 6776)
const CAST = '224.0.247.51'
const ID = crypto.randomBytes(16).toString('base64').replace(/[^\w]+/g, '')

const client = dgram.createSocket({ type: 'udp4', reuseAddr: true })

client.on('listening', () => {
  const address = client.address()
  console.log('UDP Client listening on ' + address.address + ":" + address.port)
  client.setBroadcast(true)
  client.setMulticastLoopback(true)
  client.setMulticastTTL(128)
  client.addMembership(CAST)
  message('hello')
  setInterval(() => message('ping'), 5000)
})

client.on('message', (message, remote) => {
  console.log('client | From: ' + remote.address + ':' + remote.port +' - ' + message)
})

client.bind(PORT)

function message (body) {
  const msg = Buffer.from(JSON.stringify({
    v: 0,
    agent: [pkg.name, pkg.version],
    id: ID,
    body
  }))
  client.send(msg, 0, msg.length, PORT, CAST)
  console.log(`client | sent ${body}`)
}
