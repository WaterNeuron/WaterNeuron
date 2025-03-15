# e2e Testing

Here is a brief overview of how to setup the testing environment before running the tests:

An `intermediary` account is responsible for distributing ICP/nICP to the principals used in this test.

The intermediary account has the following addresses:

- Account id: 90526bdfd692793cba1f96bde9079994ce4d40033746f04c12064ea599e2c274
- Principal: syna7-6ipnd-myx4g-ia46u-nxwok-u5nrr-yxgpi-iang7-lvru2-i7n23-tqe

To supply ICP:
dfx identity use icp-ident-RqOPnjj5ERjAEnwlvfKw
dfx ledger transfer --memo 0 --icp 1_000_000_000 90526bdfd692793cba1f96bde9079994ce4d40033746f04c12064ea599e2c274

To supply nICP:
You can run `npm run dev` then use the app and connect to an address. Look for its account id in the wallet then send ICP on it (running the command above for the corresponding address). After that, convert ICP into nICP and send the nICP to the principal of the minting account.

Run `npm run test` or `npx playwright test --project=chromium --headed`.
