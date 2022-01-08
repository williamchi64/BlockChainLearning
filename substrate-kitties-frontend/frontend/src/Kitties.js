import React, { useEffect, useState } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { useSubstrate } from './substrate-lib'
import { TxButton } from './substrate-lib/components'

import KittyCards from './KittyCards'

export default function Kitties (props) {
  const { api, keyring } = useSubstrate()
  const { accountPair } = props

  const [kittiesInform, setKittiesInform] = useState([])
  const [kitties, setKitties] = useState([])
  const [status, setStatus] = useState('')

  // hex to vec u8
  const dnaToArray = dna => {
    const dnaStr = `${dna}`
    const dnaArr = []
    for (let i = 2; i < dnaStr.length; i += 2) {
      dnaArr.push(parseInt(dnaStr[i] + dnaStr[i + 1], 16))
    }
    return dnaArr
  }
  // construct a kitty object
  const constructKitty = (id, dna, owner) => ({
    id: id,
    dna: dna,
    owner: owner
  })

  const fetchKitties = () => {
    // TODO: 在这里调用 `api.query.kittiesModule.*` 函数去取得猫咪的信息。
    // 你需要取得：
    //   - 共有多少只猫咪
    //   - 每只猫咪的 DNA 是什么，用来组合出它的形态
    //   - 每只猫咪的主人是谁
    let unsub = null
    // change state by kitties count change
    const asyncFetch = async () => {
      unsub = await api.query.kittiesModule.kittiesCount(
        async count => {
          const kittiesCount = parseInt(count)
          if (!isNaN(kittiesCount)) {
            // cannot use entries, which will collect a not sorted kitties, causing wrong id display
            const kittyEntries = await api.query.kittiesModule.kitties.multi([...Array(kittiesCount).keys()])
            const owenrEntries = await api.query.kittiesModule.owner.multi([...Array(kittiesCount).keys()])

            const dnaArr = kittyEntries.map(dnaToArray)
            const owners = owenrEntries.map(entry => `${entry}`)

            setKittiesInform([kittiesCount, dnaArr, owners])
          }
        }
      )
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  const populateKitties = () => {
    // TODO: 在这里添加额外的逻辑。你需要组成这样的数组结构：
    //  ```javascript
    //  const kitties = [{
    //    id: 0,
    //    dna: ...,
    //    owner: ...
    //  }, { id: ..., dna: ..., owner: ... }]
    //  ```
    // 这个 kitties 会传入 <KittyCards/> 然后对每只猫咪进行处理
    let unsub = null
    // change state by kitties state change on chain
    const asyncFetch = async () => {
      const kittiesCount = kittiesInform[0]
      unsub = await api.query.kittiesModule.kitties(
        kittiesCount - 1, (multiKitties) => {
          const dnaArr = kittiesInform[1]
          const owners = kittiesInform[2]
          const kitties = []
          for (let i = 0; i < kittiesCount; i++) {
            kitties.push(constructKitty(i, dnaArr[i], owners[i]))
          }
          setKitties(kitties)
        }
      )
    }

    asyncFetch()

    return () => {
      unsub && unsub()
    }
  }

  useEffect(fetchKitties, [api, keyring])
  useEffect(populateKitties, [api, kittiesInform])

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>
}
