use basset::hub::Config;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, AllBalanceResponse, Api, BalanceResponse, BankQuery,
    CanonicalAddr, Coin, ContractResult, Empty, FullDelegation, OwnedDeps, Querier, QuerierResult,
    QueryRequest, SystemError, SystemResult, Uint128, Validator, WasmQuery,
};
use cosmwasm_storage::to_length_prefixed;
use std::collections::HashMap;

use cw20::{BalanceResponse as Cw20BalanceResponse, Cw20QueryMsg};
use cw20_base::state::{MinterData, TokenInfo};

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let contract_addr = MOCK_CONTRACT_ADDR;
    let custom_querier: WasmMockQuerier =
        WasmMockQuerier::new(MockQuerier::new(&[(contract_addr, contract_balance)]));

    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: custom_querier,
        custom_query_type: Default::default(),
    }
}

pub(crate) fn _caps_to_map(caps: &[(&String, &Uint128)]) -> HashMap<String, Uint128> {
    let mut owner_map: HashMap<String, Uint128> = HashMap::new();
    for (denom, cap) in caps.iter() {
        owner_map.insert(denom.to_string(), **cap);
    }
    owner_map
}

pub struct WasmMockQuerier {
    base: MockQuerier,
    token_querier: TokenQuerier,
    balance_querier: BalanceQuerier,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        // MockQuerier doesn't support Custom, so we ignore it completely here
        let request: QueryRequest<_> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

#[allow(clippy::if_same_then_else)]
impl WasmMockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(WasmQuery::Raw { contract_addr, key }) => {
                let prefix_config = to_length_prefixed(b"config").to_vec();
                let prefix_balance = to_length_prefixed(b"balance").to_vec();
                let api: MockApi = MockApi::default();

                if key.as_slice().to_vec() == prefix_config {
                    let config = Config {
                        token_contract_registered: false,
                        token_contract: Some(api.addr_canonicalize("token").unwrap()),
                        protocol_fee_collector: None,
                        rewards_contract: None,
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(
                        &to_binary(&config).unwrap(),
                    )))
                } else if key.as_slice()[..prefix_balance.len()].to_vec() == prefix_balance {
                    let key_address: &[u8] = &key.as_slice()[prefix_balance.len()..];
                    let address_raw: CanonicalAddr = CanonicalAddr::from(key_address);
                    let balances: &HashMap<String, Uint128> =
                        match self.token_querier.balances.get(contract_addr) {
                            Some(balances) => balances,
                            None => {
                                return SystemResult::Err(SystemError::InvalidRequest {
                                    error: format!(
                                        "No balance info exists for the contract {}",
                                        contract_addr
                                    ),
                                    request: key.as_slice().into(),
                                })
                            }
                        };
                    let api: MockApi = MockApi::default();
                    let address: String = match api.addr_humanize(&address_raw) {
                        Ok(v) => v.to_string(),
                        Err(e) => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: format!("Parsing query request: {}", e),
                                request: key.as_slice().into(),
                            })
                        }
                    };
                    let balance = match balances.get(&address) {
                        Some(v) => v,
                        None => {
                            return SystemResult::Err(SystemError::InvalidRequest {
                                error: "Balance not found".to_string(),
                                request: key.as_slice().into(),
                            })
                        }
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&balance)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::AllBalances { address }) => {
                if address == &"reward".to_string() {
                    let mut coins: Vec<Coin> = vec![];
                    let luna = Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(luna);
                    let all_balances = AllBalanceResponse { amount: coins };
                    SystemResult::Ok(ContractResult::from(to_binary(&all_balances)))
                } else if address == MOCK_CONTRACT_ADDR {
                    let mut coins: Vec<Coin> = vec![];
                    let luna = Coin {
                        denom: "uluna".to_string(),
                        amount: Uint128::new(1000u128),
                    };
                    coins.push(luna);
                    let all_balances = AllBalanceResponse { amount: coins };
                    SystemResult::Ok(ContractResult::from(to_binary(&all_balances)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Bank(BankQuery::Balance { address, denom }) => {
                if address == MOCK_CONTRACT_ADDR && denom == "uluna" {
                    match self.balance_querier.balances.get(MOCK_CONTRACT_ADDR) {
                        Some(coin) => {
                            SystemResult::Ok(ContractResult::from(to_binary(&BalanceResponse {
                                amount: Coin {
                                    denom: coin.denom.clone(),
                                    amount: coin.amount,
                                },
                            })))
                        }
                        None => SystemResult::Err(SystemError::InvalidRequest {
                            error: "balance not found".to_string(),
                            request: Default::default(),
                        }),
                    }
                } else if address == &"reward".to_string() && denom == "uusd" {
                    let bank_res = BalanceResponse {
                        amount: Coin {
                            amount: Uint128::new(2000u128),
                            denom: denom.to_string(),
                        },
                    };
                    SystemResult::Ok(ContractResult::from(to_binary(&bank_res)))
                } else {
                    unimplemented!()
                }
            }
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                match from_binary(msg).unwrap() {
                    Cw20QueryMsg::TokenInfo {} => {
                        let balances: &HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances,
                                None => {
                                    return SystemResult::Err(SystemError::InvalidRequest {
                                        error: format!(
                                            "No balance info exists for the contract {}",
                                            contract_addr
                                        ),
                                        request: msg.as_slice().into(),
                                    })
                                }
                            };
                        let mut total_supply = Uint128::zero();

                        for balance in balances {
                            total_supply += *balance.1;
                        }
                        let _api: MockApi = MockApi::default();
                        let token_inf: TokenInfo = TokenInfo {
                            name: "bluna".to_string(),
                            symbol: "BLUNA".to_string(),
                            decimals: 6,
                            total_supply,
                            mint: Some(MinterData {
                                minter: Addr::unchecked("hub"),
                                cap: None,
                            }),
                        };
                        SystemResult::Ok(ContractResult::Ok(to_binary(&token_inf).unwrap()))
                    }
                    Cw20QueryMsg::Balance { address } => {
                        let balances: &HashMap<String, Uint128> =
                            match self.token_querier.balances.get(contract_addr) {
                                Some(balances) => balances,
                                None => {
                                    return SystemResult::Err(SystemError::InvalidRequest {
                                        error: format!(
                                            "No balance info exists for the contract {}",
                                            contract_addr
                                        ),
                                        request: msg.as_slice().into(),
                                    })
                                }
                            };

                        let balance = match balances.get(&address) {
                            Some(v) => *v,
                            None => {
                                return SystemResult::Ok(ContractResult::Ok(
                                    to_binary(&Cw20BalanceResponse {
                                        balance: Uint128::zero(),
                                    })
                                    .unwrap(),
                                ));
                            }
                        };

                        SystemResult::Ok(ContractResult::Ok(
                            to_binary(&Cw20BalanceResponse { balance }).unwrap(),
                        ))
                    }
                    _ => panic!("DO NOT ENTER HERE"),
                }
            }
            _ => self.base.handle_query(request),
        }
    }
    pub fn update_staking(
        &mut self,
        denom: &str,
        validators: &[Validator],
        delegations: &[FullDelegation],
    ) {
        self.base.update_staking(denom, validators, delegations);
    }
}

#[derive(Clone, Default)]
pub struct BalanceQuerier {
    balances: HashMap<String, Coin>,
}

impl BalanceQuerier {
    pub fn new(balances: &[(String, Coin)]) -> Self {
        BalanceQuerier {
            balances: native_balances_to_map(balances),
        }
    }
}

#[derive(Clone, Default)]
pub struct TokenQuerier {
    balances: HashMap<String, HashMap<String, Uint128>>,
}

impl TokenQuerier {
    pub fn new(balances: &[(&String, &[(&String, &Uint128)])]) -> Self {
        TokenQuerier {
            balances: balances_to_map(balances),
        }
    }
}

pub(crate) fn native_balances_to_map(balances: &[(String, Coin)]) -> HashMap<String, Coin> {
    let mut balances_map: HashMap<String, Coin> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let coin = Coin {
            denom: balances.clone().denom,
            amount: balances.clone().amount,
        };
        balances_map.insert(contract_addr.to_string(), coin);
    }
    balances_map
}

pub(crate) fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut balances_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (contract_addr, balances) in balances.iter() {
        let mut contract_balances_map: HashMap<String, Uint128> = HashMap::new();
        for (addr, balance) in balances.iter() {
            contract_balances_map.insert(addr.to_string(), **balance);
        }

        balances_map.insert(contract_addr.to_string(), contract_balances_map);
    }
    balances_map
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier) -> Self {
        WasmMockQuerier {
            base,
            token_querier: TokenQuerier::default(),
            balance_querier: BalanceQuerier::default(),
        }
    }

    pub fn with_native_balances(&mut self, balances: &[(String, Coin)]) {
        self.balance_querier = BalanceQuerier::new(balances);
    }

    // configure the mint whitelist mock basset
    pub fn with_token_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.token_querier = TokenQuerier::new(balances);
    }
}
