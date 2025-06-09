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

import { randomUUID } from 'node:crypto'
import { test, describe, beforeAll, assert } from 'vitest'
import { generateBytes, generateFixedBytes } from '../utils.mjs'
import { Readable, Writable } from 'node:stream'
import { finished, pipeline } from 'node:stream/promises'

/**
 * @param {import("../../index").Operator} op
 */
export function run(op) {
  describe('blocking read options', () => {
    const capability = op.capability()
    beforeAll(() => {
      assert.equal(capability.read && capability.write, true)
    })

    test.runIf(capability.readWithIfMatch)('readWithIfMatch', () => {

    })
  })
}
