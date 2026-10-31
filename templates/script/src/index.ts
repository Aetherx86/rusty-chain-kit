// {{name}}: viem script scaffolded by para. Node >= 23 runs TypeScript directly.
//   ETH_RPC_URL=$(para rpc url mainnet) npm start
//   npm run watch
import { createPublicClient, formatEther, formatGwei, http, type Address } from 'viem'

const url = process.env.ETH_RPC_URL ?? 'http://127.0.0.1:8545'
const client = createPublicClient({ transport: http(url) })

const [chainId, block, gasPrice] = await Promise.all([
  client.getChainId(),
  client.getBlock(),
  client.getGasPrice(),
])
console.log(`rpc       ${url}`)
console.log(`chain id  ${chainId}`)
console.log(`block     ${block.number} (${block.transactions.length} txs)`)
console.log(`gas price ${formatGwei(gasPrice)} gwei`)

const vitalik: Address = '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045'
console.log(`balance   ${formatEther(await client.getBalance({ address: vitalik }))} ETH (${vitalik})`)

if (process.argv.includes('--watch')) {
  client.watchBlocks({
    onBlock: (b) =>
      console.log(
        `block ${b.number} txs=${b.transactions.length} baseFee=${b.baseFeePerGas ? formatGwei(b.baseFeePerGas) : '-'} gwei`,
      ),
  })
}
