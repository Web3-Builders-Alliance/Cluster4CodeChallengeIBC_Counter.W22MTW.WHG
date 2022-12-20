use cosmwasm_std::{
    entry_point, from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, Event,
    Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg, IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg, IbcReceiveResponse, QueryRequest, StdResult, SystemResult, WasmMsg, Uint128, Addr, MessageInfo,
};

use crate::ibc_helpers::{StdAck, try_get_ack_error, validate_order_and_version};

use crate::error::ContractError;
use crate::msg::PacketMsg;
use crate::state::STATE;
//use crate::state::PENDING;
use crate::contract;

pub const IBC_VERSION: &str = "counter-1";

#[entry_point]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())
}

#[entry_point]
/// once it's established, we create the reflect contract
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    validate_order_and_version(msg.channel(), msg.counterparty_version())?;

    let mut state = STATE.load(deps.storage)?;
    if state.endpoint.is_some() {
        return Err(ContractError::AlreadyConnected {});
    }
    state.endpoint = Some(msg.channel().endpoint.clone());

    STATE.save(deps.storage, &state)?;

    Ok(IbcBasicResponse::new()
        .add_attribute("method", "ibc_channel_connect")
        .add_attribute("channel", &msg.channel().endpoint.channel_id)
        .add_attribute("port", &msg.channel().endpoint.port_id))
}

#[entry_point]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    match msg {
        // Error any TX that would cause the channel to close that is
        // coming from the local chain.
        IbcChannelCloseMsg::CloseInit { channel: _ } => Err(ContractError::CantCloseChannel {}),
        // If we're here, something has gone catastrophically wrong on
        // our counterparty chain. Per the `CloseInit` handler above,
        // this contract will _never_ allow its channel to be
        // closed.
        //
        // Note: erroring here would prevent our side of the channel
        // closing (bad because the channel is, for all intents and
        // purposes, closed) so we must allow the transaction through.
        IbcChannelCloseMsg::CloseConfirm { channel: _ } => Ok(IbcBasicResponse::default()),
        _ => unreachable!("https://github.com/CosmWasm/cosmwasm/pull/1449"),
    }
}

#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    info:MessageInfo,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let packet_msg: StdResult<PacketMsg>  = from_slice(&msg.packet.data).unwrap();
   

    match packet_msg {
        PacketMsg::Increment { } => increment(deps, env),
        PacketMsg::Reset { count } => reset(deps, env, count, info),
    }
}

pub fn increment(deps:DepsMut, env:Env) -> Result<IbcReceiveResponse, ContractError> {
    contract::execute::increment(deps, env)?;
    Ok(IbcReceiveResponse::new()
        .add_attribute("method", "ibc_packet_receive")
        .set_ack(StdAck::success(&"0")))
}

pub fn reset(deps:DepsMut, env:Env, count:i32, info:MessageInfo) -> Result<IbcReceiveResponse, ContractError> {
    contract::execute::reset(deps, info, count)?;
    Ok(IbcReceiveResponse::new()
        .add_attribute("method", "ibc_packet_receive")
        .set_ack(StdAck::success(&"0")))
}

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}
