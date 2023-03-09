use crate::state::{State, *};
use concordium_std::*;

#[derive(Debug, Serialize, SchemaType)]
struct ViewResponse {
    proj_admin: AccountAddress,
    status: SaleStatus,
    paused: bool,
    addr_ovl: Address,
    addr_bbb: Address,
    ovl_claimed_inc: u8,
    bbb_claimed_inc: u8,
    project_token: Option<ContractAddress>,
    schedule: SaleSchedule,
    saleinfo: SaleInfo,
}

#[receive(
    contract = "pub_rido_ccd",
    name = "view",
    return_value = "ViewResponse"
)]
fn contract_view<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<ViewResponse> {
    let state = host.state();

    Ok(ViewResponse {
        proj_admin: state.proj_admin,
        status: state.status.clone(),
        paused: state.paused,
        addr_ovl: state.addr_ovl,
        addr_bbb: state.addr_bbb,
        ovl_claimed_inc: state.ovl_claimed_inc,
        bbb_claimed_inc: state.bbb_claimed_inc,
        project_token: state.project_token,
        schedule: state.schedule.clone(),
        saleinfo: state.saleinfo.clone(),
    })
}

// ------------------------------------------

type ViewParticipantsResponse = Vec<(Address, UserState)>;

#[receive(
    contract = "pub_rido_ccd",
    name = "viewParticipants",
    return_value = "ViewParticipantsResponse"
)]
fn contract_view_participants<S: HasStateApi>(
    _ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<ViewParticipantsResponse> {
    let state = host.state();

    let mut ret: Vec<(Address, UserState)> = Vec::new();
    for (addr, user_state) in state.participants.iter() {
        ret.push((*addr, user_state.clone()));
    }

    Ok(ret)
}

// ------------------------------------------

#[receive(contract = "pub_rido_ccd", name = "viewWinUnits", return_value = "u8")]
fn contract_win_units<S: HasStateApi>(
    ctx: &impl HasReceiveContext,
    host: &impl HasHost<State<S>, StateApiType = S>,
) -> ReceiveResult<u8> {
    let state = host.state();
    let user = ctx.sender();

    let user_state = state
        .participants
        .get(&user)
        .ok_or(ContractError::Unauthorized)?;

    Ok(user_state.win_units)
}
