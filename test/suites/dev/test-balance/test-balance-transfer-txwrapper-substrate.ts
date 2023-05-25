// // As inspired by https://github.com/paritytech/txwrapper/blob/master/examples/polkadot.ts
// // This flow is used by some exchange partners like kraken
import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { alith, ALITH_GENESIS_LOCK_BALANCE } from "@moonwall/util";
import { createSignedTx, createSigningPayload } from "@substrate/txwrapper-core/lib/core/construct";
import { methods as substrateMethods } from "@substrate/txwrapper-substrate";
import { getRegistryBase } from "@substrate/txwrapper-core/lib/core/metadata";
import { getSpecTypes, TypeRegistry } from "@substrate/txwrapper-core";
import { customDevRpcRequest, signWith } from "../../../helpers/common.js";
import { checkBalance } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

describeSuite({
  id: "D0305",
  title: "Balance transfer - TxWrapper",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let randomAddress: `0x${string}`;

    beforeAll(async () => {
      const privateKey = generatePrivateKey();
      randomAddress = privateKeyToAccount(privateKey).address;
      await context.createBlock();
      const [
        { block },
        blockHash,
        genesisHash,
        metadataRpc,
        { specVersion, transactionVersion, specName },
      ] = await Promise.all([
        customDevRpcRequest("chain_getBlock"),
        customDevRpcRequest("chain_getBlockHash"),
        customDevRpcRequest("chain_getBlockHash", [0]),
        customDevRpcRequest("state_getMetadata"),
        customDevRpcRequest("state_getRuntimeVersion"),
      ]);

      const registry = getRegistryBase({
        chainProperties: {
          ss58Format: 1285,
          tokenDecimals: 18,
          tokenSymbol: "MOVR",
        },
        specTypes: getSpecTypes(new TypeRegistry(), "Moonriver", specName, specVersion),
        metadataRpc,
      });

      const unsigned = substrateMethods.balances.transfer(
        {
          dest: randomAddress as any,
          value: 512,
        },
        {
          address: alith.address,
          blockHash,
          blockNumber: registry.createType("BlockNumber", block.header.number).toNumber(),
          eraPeriod: 64,
          genesisHash,
          metadataRpc,
          nonce: 0, // Assuming this is Alith's first tx on the chain
          specVersion,
          tip: 0,
          transactionVersion,
        },
        {
          metadataRpc,
          registry,
        }
      );

      const signingPayload = createSigningPayload(unsigned, { registry });
      const signature = signWith(alith, signingPayload, {
        metadataRpc,
        registry,
      });
      // Serialize a signed transaction.
      const tx = createSignedTx(unsigned, signature, { metadataRpc, registry });

      await customDevRpcRequest("author_submitExtrinsic", [tx]);
      await context.createBlock();
    }, 10000);

    it({
      id: "T01",
      title: "should show the reducible balanced when some amount is locked",
      test: async function () {
        expect(await checkBalance(context, randomAddress, 1n)).toBe(0n);
        expect(await checkBalance(context, randomAddress, 2n)).toBe(512n);
      },
    });

    it({
      id: "T02",
      title: "should reflect balance identically on polkadot/web3",
      test: async function () {
        const balance = await context.polkadotJs().query.system.account(alith.address);
        expect(await checkBalance(context)).to.equal(
          balance.data.free.toBigInt() - ALITH_GENESIS_LOCK_BALANCE
        );
      },
    });

    it({
      id: "T03",
      title: "should check fees",
      test: async function () {
        await verifyLatestBlockFees(context, 512n);
      },
    });
  },
});