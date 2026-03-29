import { 
  Contract, 
  SorobanRpc, 
  xdr, 
  BASE_FEE, 
  TransactionBuilder, 
  Server 
} from '@stellar/stellar-sdk';

import type { Address } from 'soroban-sdk';

export interface FundingPreview {
  trade_id: bigint;
  buyer: string;
  seller: string;
  amount: bigint;
  fee: bigint;
  buyer_balance: bigint;
  allowance_sufficient: boolean;
}

export class StellarEscrowClient {
  private server: Server;
  private contract: Contract;
  private rpcServer: SorobanRpc.Server;

  constructor(
    network: 'testnet' | 'mainnet',
    contractId: string,
    rpcUrl: string = 'https://soroban-testnet.stellar.org',
    horizonUrl: string = 'https://horizon-testnet.stellar.org'
  ) {
    this.server = new Server(horizonUrl);
    this.rpcServer = new SorobanRpc.Server(rpcUrl);
    this.contract = new Contract(contractId);
  }

  async getFundingPreview(
    tradeId: number,
    buyer: string,
    sourceKeypair: string // For auth
  ): Promise<FundingPreview> {
    const account = await this.server.getAccount(sourceKeypair);
    const tx = new TransactionBuilder(account, { fee: BASE_FEE })
      .addInvocation(this.contract.call('get_funding_preview', 
        xdr.ScVal.scvU64(tradeId),
        xdr.ScVal.scvAddress(xdr.Address.fromString(buyer))
      ))
      .setTimeout(30)
      .setNetworkPassphrase(network === 'mainnet' ? '...' : 'Test SDF Network ; September 2015')
      .build();

    // Simulate first
    const sim = await this.rpcServer.simulateTransaction(tx);
    if (SorobanRpc.Api.isSimulationError(sim.results[0])) {
      throw new Error('Simulation failed');
    }

    // Send & restore
    const txHash = await this.rpcServer.sendTransaction(tx);
    const result = await this.rpcServer.restoreTransaction(txHash);
    
    return result.results[0].xdrResult!.toXDR().read().result() as unknown as FundingPreview;
  }

  async fundTrade(
    tradeId: number,
    buyerKeypair: string,
    preview: FundingPreview
  ): Promise<string> {
    const account = await this.server.getAccount(buyerKeypair);
    const tx = new TransactionBuilder(account, { fee: BASE_FEE })
      .addInvocation(this.contract.call('execute_fund',
        xdr.ScVal.scvU64(tradeId),
        xdr.ScVal.scvAddress(xdr.Address.fromString(preview.buyer)),
        // Preview ScVal conversion needed
        this.previewToScVal(preview)
      ))
      .setTimeout(30)
      .setNetworkPassphrase('Test SDF Network ; September 2015')
      .build();

    tx.sign(keypair); // Sign with buyer keypair
    const txHash = await this.rpcServer.sendTransaction(tx);
    return txHash;
  }

  private previewToScVal(preview: FundingPreview): xdr.ScVal {
    // Complex ScVal conversion for FundingPreview struct
    // This is simplified - full impl needs xdr struct matching
    return xdr.ScVal.scvVoid();
  }

  async getTradeDetail(tradeId: number): Promise<any> {
    // Similar pattern for get_trade_detail
    // Implement based on backend TradeDetail
  }
}

export const escrowClient = new StellarEscrowClient('testnet', 'YOUR_CONTRACT_ID');
