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

import { checkRandomRootEnabled, generateRandomRoot, loadConfigFromEnv, loadTestSchemeFromEnv } from './tests/utils.mjs'
import { Operator, layers } from './index.mjs'

export default async function setup() {
  // const { vi } = await import('vitest')
  const scheme = loadTestSchemeFromEnv()
  if (!scheme) {
    console.warn('The scheme is empty. Test will be skipped.')
    // return
  }

  const config = loadConfigFromEnv(scheme)

  if (checkRandomRootEnabled()) {
    if (config.root) {
      config.root = generateRandomRoot(config.root)
    } else {
      console.warn("The root is not set. Won't generate random root.")
    }
  }

  console.log('scheme', scheme)

  let operator = scheme ? new Operator(scheme, config) : null
  console.log('QAQ operator', operator)

  let retryLayer = new layers.RetryLayer()
  retryLayer.jitter = true
  retryLayer.maxTimes = 4

  if (operator) {
    // vi.stubGlobal('operator', operator)
    globalThis.operator = operator
  }

  return () => {
    // vi.unstubAllGlobals()
    globalThis.operator = null
  }
}
