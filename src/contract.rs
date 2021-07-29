use cosmwasm_std::{
    log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, StdError, StdResult, Storage,
};

use crate::msg::{AddressResponse, HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        receive_addr: msg.receive_addr,
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SendPayment { pay } => send_payment(deps, env, pay),
        HandleMsg::ResetAddr { address } => reset_addr(deps, env, address),
    }
}

pub fn send_payment<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    pay: Vec<Coin>,
) -> StdResult<HandleResponse> {
    if _env.message.sent_funds.is_empty() || _env.message.sent_funds[0].denom != "ust" {
        return Err(StdError::generic_err(
            "You must pass some ust coins to send make a multisend",
        ));
    }
    let state = config_read(&deps.storage).load()?;
    let mut messages: Vec<CosmosMsg> = Vec::new();
    let from_address = _env.contract.address.clone();
    let to_address = state.receive_addr;
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        from_address,
        to_address,
        amount: pay,
    }));

    let r = HandleResponse {
        messages,
        log: vec![],
        data: None,
    };
    Ok(r)
}

pub fn reset_addr<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let api = &deps.api;
    config(&mut deps.storage).update(|mut state| {
        if api.canonical_address(&env.message.sender)? != state.owner {
            return Err(StdError::unauthorized());
        }
        state.receive_addr = address;
        Ok(state)
    })?;
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAddr {} => to_binary(&query_count(deps)?),
    }
}

fn query_count<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<AddressResponse> {
    let state = config_read(&deps.storage).load()?;
    Ok(AddressResponse {
        address: state.receive_addr,
    })
}
