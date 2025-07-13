export const DEV = import.meta.env ? import.meta.env.DEV : true;
export const STAGING = process.env.CANISTER_ID === '47pxu-byaaa-aaaap-ahpsa-cai';

export const HOST = DEV ? 'http://127.0.1:8080' : 'https://ic0.app';
export const DAPP_DERIVATION_ORIGIN = 'https://n3i53-gyaaa-aaaam-acfaq-cai.icp0.io';
export const IDENTITY_PROVIDER = 'https://identity.ic0.app';

export const OISY_RPC = 'https://oisy.com/sign';
export const NFID_RPC = 'https://nfid.one/rpc';

export const CANISTER_ID_II = DEV ? 'iidmm-fiaaa-aaaaq-aadmq-cai' : 'rdmx6-jaaaa-aaaaa-aaadq-cai';
export const CANISTER_ID_WTN_LEDGER = 'jcmow-hyaaa-aaaaq-aadlq-cai';
export const CANISTER_ID_ICP_LEDGER = 'ryjl3-tyaaa-aaaaa-aaaba-cai';
export const CANISTER_ID_NICP_LEDGER = 'buwm7-7yaaa-aaaar-qagva-cai';
export const CANISTER_ID_BOOMERANG = 'daijl-2yaaa-aaaar-qag3a-cai';
export const CANISTER_ID_WATER_NEURON = 'tsbvt-pyaaa-aaaar-qafva-cai';
export const CANISTER_ID_ICPSWAP_POOL = 'e5a7x-pqaaa-aaaag-qkcga-cai';
