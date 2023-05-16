import { AccessListish } from "@ethersproject/transactions";
import { RlpStructuredData, TransactionRequest, ethers } from "ethers";
import * as RLP from "rlp";
import { Contract } from "web3-eth-contract";

import {
  alith,
  ALITH_PRIVATE_KEY,
  ALITH_ADDRESS,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_PRIVATE_KEY,
  dorothy,
  DOROTHY_PRIVATE_KEY,
  ethan,
  ETHAN_PRIVATE_KEY,
} from "@moonwall/util";
import { getCompiled } from "./contracts.js";
// import { customWeb3Request } from "./providers";
import { customDevRpcRequest } from "./common.js";
import { DevModeContext, MoonwallContext, EthTransactionType } from "@moonwall/cli";
import { expectEVMResult } from "./eth-transactions.js";

// Ethers is used to handle post-london transactions
import type { ApiPromise } from "@polkadot/api";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { FMT_BYTES, FMT_NUMBER } from "web3";
const debug = require("debug")("test:transaction");

export const DEFAULT_TXN_MAX_BASE_FEE = 10_000_000_000;

export interface TransactionOptions {
  from?: string;
  to?: string;
  privateKey?: string;
  nonce?: number;
  gas?: string | number;
  gasPrice?: string | number | BigInt;
  maxFeePerGas?: string | number | BigInt;
  maxPriorityFeePerGas?: string | number | BigInt;
  value?: string | number;
  data?: string;
  accessList?: AccessListish; // AccessList | Array<[string, Array<string>]>
}

export const TRANSACTION_TEMPLATE: TransactionOptions = {
  // nonce: null,
  gas: 500_000,
  value: "0x00",
};

export const ALITH_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: alith.address,
  privateKey: ALITH_PRIVATE_KEY,
};

export const BALTATHAR_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: baltathar.address,
  privateKey: BALTATHAR_PRIVATE_KEY,
};

export const CHARLETH_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: charleth.address,
  privateKey: CHARLETH_PRIVATE_KEY,
};

export const DOROTHY_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: dorothy.address,
  privateKey: DOROTHY_PRIVATE_KEY,
};

export const ETHAN_TRANSACTION_TEMPLATE: TransactionOptions = {
  ...TRANSACTION_TEMPLATE,
  from: ethan.address,
  privateKey: ETHAN_PRIVATE_KEY,
};

export const createTransaction = async (
  context: DevModeContext,
  options: TransactionOptions,
  txType?: EthTransactionType
): Promise<string> => {
  const defaultTxnStyle = MoonwallContext.getContext()!.defaultEthTxnStyle;

  const isLegacy = txType
    ? txType === "Legacy"
    : defaultTxnStyle
    ? defaultTxnStyle === "Legacy"
    : true;

  const isEip2930 = txType
    ? txType === "EIP2930"
    : defaultTxnStyle
    ? defaultTxnStyle === "EIP2930"
    : true;

  const isEip1559 = txType
    ? txType === "EIP1559"
    : defaultTxnStyle
    ? defaultTxnStyle === "EIP1559"
    : true;

  // a transaction shouldn't have both Legacy and EIP1559 fields
  if (options.gasPrice && options.maxFeePerGas) {
    throw new Error(`txn has both gasPrice and maxFeePerGas!`);
  }
  if (options.gasPrice && options.maxPriorityFeePerGas) {
    throw new Error(`txn has both gasPrice and maxPriorityFeePerGas!`);
  }

  // convert any bigints to hex
  if (typeof options.gasPrice === "bigint") {
    options.gasPrice = "0x" + options.gasPrice.toString(16);
  }
  if (typeof options.maxFeePerGas === "bigint") {
    options.maxFeePerGas = "0x" + options.maxFeePerGas.toString(16);
  }
  if (typeof options.maxPriorityFeePerGas === "bigint") {
    options.maxPriorityFeePerGas = "0x" + options.maxPriorityFeePerGas.toString(16);
  }

  let maxFeePerGas;
  let maxPriorityFeePerGas;
  if (options.gasPrice) {
    maxFeePerGas = options.gasPrice;
    maxPriorityFeePerGas = options.gasPrice;
  } else {
    maxFeePerGas =
      options.maxFeePerGas || (await context.ethersSigner().provider?.getFeeData())!.gasPrice;
    maxPriorityFeePerGas = options.maxPriorityFeePerGas || 0;
  }

  const gasPrice =
    options.gasPrice !== undefined
      ? options.gasPrice
      : "0x" + (await context.web3().eth.getGasPrice({number: FMT_NUMBER.HEX, bytes: FMT_BYTES.HEX}));
  const value = options.value !== undefined ? options.value : "0x00";
  const from = options.from || alith.address;
  const privateKey = options.privateKey !== undefined ? options.privateKey : ALITH_PRIVATE_KEY;

  // Allows to retrieve potential errors
  let error = "";
  const estimatedGas = await context
    .web3().eth
    .estimateGas({
      from: from,
      to: options.to,
      data: options.data,
    })
    .catch((e) => {
      error = e;
      return options.gas || 12_500_000;
    });

  let warning = "";
  if (options.gas && options.gas < estimatedGas) {
    warning = `Provided gas ${options.gas} is lower than estimated gas ${estimatedGas}`;
  }
  // Instead of hardcoding the gas limit, we estimate the gas
  const gas = options.gas || estimatedGas;

  const accessList = options.accessList || [];
  const nonce =
    options.nonce != null
      ? options.nonce
      : await context.web3().eth.getTransactionCount(from, "pending")
      // : await context.ethersSigner().provider!.getTransactionCount(from, "pending");

  let data, rawTransaction;
  const provider = context.ethersSigner().provider!;
  // const provider = context.web3().provider
  // const newSigner = new ethers.Wallet(privateKey, provider);
  if (isLegacy) {
    data = {
      from,
      to: options.to,
      value: value && value.toString(),
      gasPrice,
      gas,
      nonce: nonce,
      data: options.data,
    };
    // rawTransaction = await newSigner.signTransaction(data);
    // rawTransaction = await context.web3().eth.signTransaction(data);
    const tx = await context.web3().eth.accounts.signTransaction(data as any, privateKey);
    rawTransaction = tx.rawTransaction;
  } else {
    const signer = new ethers.Wallet(privateKey, context.ethersSigner().provider!);
    const chainId = (await provider.getNetwork()).chainId;
    // const chainId = await context.web3().eth.getChainId()
    if (isEip2930) {
      data = {
        from,
        to: options.to,
        value: value && value.toString(),
        gasPrice,
        gasLimit: gas,
        nonce: nonce,
        data: options.data,
        accessList,
        chainId,
        type: 1,
      };
    } else  {
      if (!isEip1559){
        throw new Error("Unknown transaction type!");
      }

      data = {
        from,
        to: options.to,
        value: value && value.toString(),
        maxFeePerGas,
        maxPriorityFeePerGas,
        gasLimit: gas,
        nonce: nonce,
        data: options.data,
        accessList,
        chainId,
        type: 2,
      };
    }
    // rawTransaction = await newSigner.signTransaction(data as TransactionRequest);
    rawTransaction = await signer.signTransaction(data as any);
  }

  debug(
    `TransactionDetails` +
      (data.to
        ? `to: ${data.to.substr(0, 5) + "..." + data.to.substr(data.to.length - 3)}, `
        : "") +
      (data.value ? `value: ${data.value.toString()}, ` : "") +
      (data.gasPrice ? `gasPrice: ${data.gasPrice.toString()}, ` : "") +
      (data.maxFeePerGas ? `maxFeePerGas: ${data.maxFeePerGas.toString()}, ` : "") +
      (data.maxPriorityFeePerGas
        ? `maxPriorityFeePerGas: ${data.maxPriorityFeePerGas.toString()}, `
        : "") +
      (data.accessList ? `accessList: ${data.accessList.toString()}, ` : "") +
      (data.gas ? `gas: ${data.gas.toString()}, ` : "") +
      (data.nonce ? `nonce: ${data.nonce.toString()}, ` : "") +
      (!data.data
        ? ""
        : `data: ${
            data.data.length < 50
              ? data.data
              : data.data.substr(0, 5) + "..." + data.data.substr(data.data.length - 3)
          }, `) +
      (error ? `ERROR: ${error.toString()}, ` : "") +
      (warning ? `WARN: ${warning.toString()}, ` : "")
  );
  return rawTransaction;
};

export const createTransfer = async (
  context: DevModeContext,
  to: string,
  value: number | string | BigInt,
  options: TransactionOptions = ALITH_TRANSACTION_TEMPLATE
): Promise<string> => {
  return await createTransaction(context, {
    ...options,
    value: value.toString(),
    to,
  });
};

// Will create the transaction to deploy a contract.
// This requires to compute the nonce. It can't be used multiple times in the same block from the
// same from
// export async function createContract(
//   context: DevModeContext,
//   contractName: string,
//   options: TransactionOptions = { ...ALITH_TRANSACTION_TEMPLATE, gas: 5_000_000 },
//   contractArguments: any[] = []
// ): Promise<{ rawTx: string; contract: Contract; contractAddress: string }> {
//   const contractCompiled = getCompiled(contractName);
//   const from = options.from !== undefined ? options.from : alith.address;
//   const nonce = options.nonce || (await context.ethersSigner().getNonce());
//   // const contractAddress =
//   //   "0x" +
//   //   ethers
//   //     .keccak256(RLP.encode([from, nonce]))
//   //     .slice(12)
//   //     .substring(14);

//   // const contract = new ethers.Contract(
//   //   contractAddress,
//   //   contractCompiled.contract.abi,
//   //   context.ethersSigner()
//   // );
//     // const data = contract.de

//     const factory = new ethers.ContractFactory(contractCompiled.contract.abi, contractCompiled.byteCode, context.ethersSigner());
//     const contract = await factory.deploy(...contractArguments);
//     // const contractAddress = contract;
//     await contract.waitForDeployment()
//   const rawTx = contract.getDeployedCode()
//     const {con} =contract.deploymentTransaction()!.raw

//   // const contract = new (context.web3()).eth.Contract(contractCompiled.contract.abi, contractAddress);
//   // const data = contract
//   //   .deploy({
//   //     data: contractCompiled.byteCode,
//   //     arguments: contractArguments,
//   //   })
//   //   .encodeABI();

//   // const rawTx = await createTransaction(context, { ...options, from, nonce, data });

//   return {
//     rawTx,
//     contract,
//     contractAddress,
//   };
// }

// Will create the transaction to execute a contract function.
// This requires to compute the nonce. It can't be used multiple times in the same block from the
// same from
export async function createContractExecution(
  context: DevModeContext,
  execution: {
    contract: Contract;
    contractCall: any;
  },
  options: TransactionOptions = {
    from: alith.address,
    privateKey: ALITH_PRIVATE_KEY,
  }
) {
  const rawTx = await createTransaction(context, {
    ...options,
    to: execution.contract.options.address,
    data: execution.contractCall.encodeABI(),
  });

  return rawTx;
}

/**
 * Send a JSONRPC request to the node at http://localhost:9933.
 *
 * @param method - The JSONRPC request method.
 * @param params - The JSONRPC request params.
 */
export function rpcToLocalNode(rpcPort: number, method: string, params: any[] = []): Promise<any> {
  return fetch("http://localhost:" + rpcPort, {
    body: JSON.stringify({
      id: 1,
      jsonrpc: "2.0",
      method,
      params,
    }),
    headers: {
      "Content-Type": "application/json",
    },
    method: "POST",
  })
    .then((response) => response.json())
    .then(({ error, result }) => {
      if (error) {
        throw new Error(`${error.code} ${error.message}: ${JSON.stringify(error.data)}`);
      }

      return result;
    });
}
// The parameters passed to the function are assumed to have all been converted to hexadecimal
export async function sendPrecompileTx(
  context: DevModeContext,
  precompileContractAddress: string,
  selectors: { [key: string]: string },
  from: string,
  privateKey: string,
  selector: string,
  parameters: string[]
) {
  let data: string;
  if (selectors[selector]) {
    data = `0x${selectors[selector]}`;
  } else {
    throw new Error(`selector doesn't exist on the precompile contract`);
  }
  parameters.forEach((para: string) => {
    data += para.slice(2).padStart(64, "0");
  });

  return context.createBlock(
    createTransaction(context, {
      from,
      privateKey,
      value: "0x0",
      gas: "0x200000",
      gasPrice: ALITH_TRANSACTION_TEMPLATE.gasPrice,
      to: precompileContractAddress,
      data,
    })
  );
}

const GAS_PRICE = "0x" + DEFAULT_TXN_MAX_BASE_FEE.toString(16);
export async function callPrecompile(
  context: DevModeContext,
  precompileContractAddress: string,
  selectors: { [key: string]: string },
  selector: string,
  parameters: string[]
) {
  let data: string;
  if (selectors[selector]) {
    data = `0x${selectors[selector]}`;
  } else {
    throw new Error(`selector doesn't exist on the precompile contract`);
  }
  parameters.forEach((para: string) => {
    data += para.slice(2).padStart(64, "0");
  });

  return await customDevRpcRequest( "eth_call", [
    {
      from: alith.address,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: precompileContractAddress,
      data,
    },
  ]);
}

export const sendAllStreamAndWaitLast = async (
  api: ApiPromise,
  extrinsics: SubmittableExtrinsic[],
  { threshold = 500, batch = 200, timeout = 120000 } = {
    threshold: 500,
    batch: 200,
    timeout: 120000,
  }
) => {
  let promises: any[] = [];
  while (extrinsics.length > 0) {
    const pending = await api.rpc.author.pendingExtrinsics();
    if (pending.length < threshold) {
      const chunk = extrinsics.splice(0, Math.min(threshold - pending.length, batch));
      // console.log(`Sending ${chunk.length}tx (${extrinsics.length} left)`);
      promises.push(
        Promise.all(
          chunk.map((tx) => {
            return new Promise(async (resolve, reject) => {
              let unsub: () => void;
              const timer = setTimeout(() => {
                reject(`timed out`);
                unsub();
              }, timeout);
              unsub = await tx.send((result) => {
                // reset the timer
                if (result.isError) {
                  console.log(result.toHuman());
                  clearTimeout(timer);
                  reject(result.toHuman());
                }
                if (result.isInBlock) {
                  unsub();
                  clearTimeout(timer);
                  resolve(null);
                }
              });
            }).catch((e) => {});
          })
        )
      );
    }
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }
  await Promise.all(promises);
};

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;
// export const setupErc20Contract = async (context: DevModeContext, name: string, symbol: string) => {
//   const { contract, contractAddress, rawTx } = await createContract(
//     context,
//     "ERC20WithInitialSupply",
//     {
//       ...ALITH_TRANSACTION_TEMPLATE,
//       gas: 5_000_000,
//     },
//     [name, symbol, ALITH_ADDRESS, ERC20_TOTAL_SUPPLY]
//   );
//   const { result } = await context.createBlock(rawTx);
//   expectEVMResult(result.events, "Succeed");
//   return { contract, contractAddress };
// };