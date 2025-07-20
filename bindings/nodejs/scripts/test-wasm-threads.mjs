#!/usr/bin/env node

/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

import { spawn } from 'child_process'
import { existsSync, statSync } from 'fs'
import path from 'path'

console.log('Testing wasm32-wasip1-threads target support...')

// Set up environment
const env = {
  ...process.env,
  EMNAPI_LINK_DIR: path.resolve(process.cwd(), 'node_modules/.pnpm/emnapi@1.4.4/node_modules/emnapi/lib/wasm32-wasip1-threads')
}

// Build for wasm32-wasip1-threads
console.log('Building for wasm32-wasip1-threads...')
const buildProcess = spawn('cargo', ['build', '--target', 'wasm32-wasip1-threads', '--release'], {
  stdio: 'inherit',
  env
})

buildProcess.on('exit', (code) => {
  if (code !== 0) {
    console.error('Build failed!')
    process.exit(1)
  }

  // Check if WASM file was generated
  const wasmPath = 'target/wasm32-wasip1-threads/release/opendal_nodejs.wasm'
  if (!existsSync(wasmPath)) {
    console.error('WASM file not found!')
    process.exit(1)
  }

  const stats = statSync(wasmPath)
  console.log(`✅ Successfully built WASM file: ${wasmPath}`)
  console.log(`📦 File size: ${(stats.size / 1024 / 1024).toFixed(2)} MB`)
  console.log('🎉 wasm32-wasip1-threads target is supported with async/await!')
})

buildProcess.on('error', (err) => {
  console.error('Build process error:', err)
  process.exit(1)
})
