/**
 * TODO: Automate the entire process with a GitHub Action:
 * 1. Trigger when changes are detected in the runtime code.
 * 2. Execute this script to generate updated types.
 * 3. Open a pull request to the `main` branch with the changes.
 * 4. Upon merging the PR, the automated publishing workflow
 *    will publish the updated package.
 */

// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

const { exec, execSync, spawn } = require('node:child_process')
const { readFileSync } = require('node:fs')
const { join, resolve } = require('node:path')

const CWD = process.cwd()
const TYPES_DIR = join(CWD, 'types')
const METADATA_PATH = resolve(CWD, 'types/src/metadata.json')
const ENDPOINT = 'http://127.0.0.1:9944'

let nodeProcess

function runTangleNode() {
  console.log('Starting Tangle node...')
  nodeProcess = spawn('sh', ['./scripts/run-standalone-local.sh', '--clean'], {
    stdio: ['ignore', 'ignore', 'ignore'],
    detached: true,
  })

  nodeProcess.on('error', error => {
    console.error('Failed to start Tangle node:', error)
  })

  return nodeProcess
}

function stopTangleNode() {
  if (nodeProcess) {
    console.log('\n🛑 Stopping Tangle node...')
    // Kill the entire process group
    process.kill(-nodeProcess.pid)
  }
}

// Centralized error handling
function handleError(message, error) {
  console.error(`❌ ${message}:`, error.message)
  throw error
}

// Utility functions
const sleep = seconds =>
  new Promise(resolve => setTimeout(resolve, seconds * 1000))

const logStep = message => console.log(`\n${message}`)

const runCommand = (command, directory = TYPES_DIR) => {
  logStep(`🔧 Running \`${command}\` in the \`${directory}\` directory...`)
  try {
    execSync(command, { cwd: directory, stdio: 'ignore' })
    console.log(`✅ \`${command}\` completed successfully`)
  } catch (error) {
    handleError(`Error running \`${command}\``, error)
  }
}

function deepCompare(obj1, obj2) {
  // Check if both are of the same type
  if (typeof obj1 !== typeof obj2) {
    return false
  }

  // If they're not objects, compare directly
  if (typeof obj1 !== 'object' || obj1 === null || obj2 === null) {
    return obj1 === obj2
  }

  // Get keys of both objects
  const keys1 = Object.keys(obj1)
  const keys2 = Object.keys(obj2)

  // Check if they have the same number of keys
  if (keys1.length !== keys2.length) {
    return false
  }

  // Compare each key-value pair recursively
  for (let key of keys1) {
    if (!keys2.includes(key) || !deepCompare(obj1[key], obj2[key])) {
      return false
    }
  }

  return true
}

// Metadata functions
function getCurrentMetadata() {
  logStep(`Reading the current metadata from ${METADATA_PATH}`)
  return JSON.parse(readFileSync(METADATA_PATH, 'utf8'))
}

function fetchMetadata(retryTimes = 3, retryDelaySecond = 5) {
  logStep('Fetching metadata from the node...')
  return new Promise((resolve, reject) => {
    const fetchAttempt = attemptsLeft => {
      exec(
        `curl -H "Content-Type: application/json" -d \'{"id":"1","jsonrpc":"2.0","method":"state_getMetadata","params":[]}\' ${ENDPOINT}`,
        (error, stdout) => {
          if (error) {
            if (attemptsLeft > 0) {
              console.log(
                `Fetch attempt failed. Retrying in ${retryDelaySecond} seconds... (${attemptsLeft} attempts left)`
              )
              setTimeout(
                () => fetchAttempt(attemptsLeft - 1),
                retryDelaySecond * 1000
              )
            } else {
              reject(error)
            }
            return
          }

          resolve(JSON.parse(stdout.toString()))
        }
      )
    }

    fetchAttempt(retryTimes)
  })
}

async function generateNewTypes() {
  const commands = [
    'yarn install',
    'yarn update:metadata',
    'yarn build:interfaces',
    'yarn build',
  ]

  for (const command of commands) {
    runCommand(command)
  }

  logStep('✨ New TypeScript types generated successfully! 🎉')
}

async function main() {
  runTangleNode()
  await sleep(15)

  const [metadataFromNode, currentMetadata] = await Promise.all([
    fetchMetadata(),
    getCurrentMetadata(),
  ])

  if (deepCompare(metadataFromNode, currentMetadata)) {
    logStep('ℹ️️ No change in the metadata, no need to generate new types')
    return
  }

  logStep('📝 Metadata changes detected. Generating new TypeScript types...')
  await generateNewTypes()
}

process.on('SIGINT', () => {
  stopTangleNode()
  process.exit()
})

process.on('SIGTERM', () => {
  stopTangleNode()
  process.exit()
})

process.on('exit', () => {
  stopTangleNode()
  process.exit()
})

main()
  .then(() => process.exit(0))
  .catch(error => {
    console.error('Error in the `main()` function:', error)
    process.kill(process.pid, 'SIGTERM')
  })
