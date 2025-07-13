# Local Deployment

To deploy the frontend run:

```
dfx start --clean
./local_deploy.sh
npm run dev
```

**Warning**: The fast unstake is not available because the canister cannot be deployed easily. Feel free to look at the code of ICPSwap to handle the local deployment.

# Playwright Tests

To test the frontend run (assuming you already deployed the canisters):

```
./test_init.sh
npm run test
```
