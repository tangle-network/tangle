// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
// SPDX-License-Identifier: Apache-2.0

import WebSocket from 'ws'
import fs from 'fs'

const main = (): void => {
  const endpoint = 'ws://127.0.0.1:9944'

  console.log('Connecting to ', endpoint)
  const ws = new WebSocket(endpoint)

  ws.on('error', error => {
    console.error('WebSocket error:', error)
    process.exit(1)
  })

  ws.on('open', () => {
    ws.send(
      '{"id":"1","jsonrpc":"2.0","method":"state_getMetadata","params":[]}'
    )
    console.log('data send!')
  })

  ws.on('message', msg => {
    fs.writeFileSync('./src/metadata.json', msg.toString())

    console.log('Done')
    process.exit(0)
  })
}

main()
