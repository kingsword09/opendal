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

import path from 'path'
import fs from 'fs'

console.log(__dirname)
const abslutePathBindingList = fs
  .readdirSync('.')
  .filter((p) => {
    return p.endsWith('.node')
  })
  .map((p) => {
    const [_, platform] = p.split('.')
    return {
      platform: platform,
      path: path.join(__dirname, '..', p),
      fileName: p,
    }
  })

abslutePathBindingList.forEach((bindingInfo) => {
  const npmPath = path.join(__dirname, '../../../npm')
  const packagePath = path.join(npmPath, bindingInfo.platform)
  fs.copyFileSync(bindingInfo.path, path.join(packagePath, bindingInfo.fileName))
})
