import { Connection, Keypair, clusterApiUrl } from '@solana/web3.js'
import fs from 'fs'
import os from 'os'
import { parse as yamlParse, stringify as yamlStringify } from 'yaml'

export const PAYER = loadKeypairFromFile(os.homedir() + '/.config/solana/id.json')
export const PROGRAM_WALLETS = loadKeypairFromFile('./programs/wallets/target/deploy/program_nautilus-keypair.json')
export const PROGRAM_TOKENS = loadKeypairFromFile('./programs/tokens/target/deploy/program_nautilus-keypair.json')
export const PROGRAM_RECORDS = loadKeypairFromFile('./programs/records/target/deploy/program_nautilus-keypair.json')
export const PROGRAM_ACCOUNTS = loadKeypairFromFile('./programs/accounts/target/deploy/program_nautilus-keypair.json')

function loadKeypairFromFile(path: string): Keypair {
    return Keypair.fromSecretKey(
        Buffer.from(JSON.parse(fs.readFileSync(path, "utf-8")))
    )
}

const sleepSeconds = async (s: number) => await new Promise(f => setTimeout(f, s * 1000))

type TestConfigs = {
    connection: Connection,
    sleep: () => Promise<unknown>,
    skipMetadata: boolean,
}

function getTestConfigs(): TestConfigs {
    const config = yamlParse(
        fs.readFileSync(os.homedir() + '/.config/solana/cli/config.yml', "utf-8")
    )
    const jsonRpcUrl: string = config['json_rpc_url']
    const [timeDelay, skipMetadata] = jsonRpcUrl == "http://localhost:8899" ? [0 , true] : [10, false]
    return { 
        connection: new Connection(jsonRpcUrl, "confirmed"), 
        sleep: () => sleepSeconds(timeDelay), 
        skipMetadata, 
    }
}

export const TEST_CONFIGS = getTestConfigs()