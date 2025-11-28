use pentardp_rs::pdu::mcs::{
    AttachUserConfirm, AttachUserRequest, ChannelJoinConfirm, ChannelJoinRequest,
    ConnectInitial, ConnectResponse, DomainParameters, ErectDomainRequest, McsResult,
};
use pentardp_rs::pdu::tpkt::TpktPacket;
use pentardp_rs::pdu::x224::connection::{ConnectionConfirm, ConnectionRequest, Protocol};
use pentardp_rs::pdu::x224::DataPdu;
use pentardp_rs::pdu::Pdu;
use std::io::Cursor;

/// 전체 RDP 연결 시퀀스 시뮬레이션 테스트
#[test]
fn test_rdp_connection_sequence() {
    // 1. TPKT + X.224 Connection Request
    let x224_cr = ConnectionRequest::new(0x1234)
        .with_cookie("testuser")
        .with_negotiation(Protocol::Ssl as u32);

    let mut x224_cr_buffer = Vec::new();
    x224_cr.encode(&mut x224_cr_buffer).unwrap();

    let tpkt_cr = TpktPacket::new(x224_cr_buffer.clone());
    let mut tpkt_cr_buffer = Vec::new();
    tpkt_cr.encode(&mut tpkt_cr_buffer).unwrap();

    // 디코딩 검증
    let mut cursor = Cursor::new(&tpkt_cr_buffer);
    let decoded_tpkt = TpktPacket::decode(&mut cursor).unwrap();
    assert_eq!(decoded_tpkt.payload(), &x224_cr_buffer[..]);

    let mut payload_cursor = Cursor::new(decoded_tpkt.payload());
    let decoded_x224_cr = ConnectionRequest::decode(&mut payload_cursor).unwrap();
    assert_eq!(decoded_x224_cr.cookie(), Some("Cookie: mstshash=testuser\r\n"));

    // 2. TPKT + X.224 Connection Confirm
    let x224_cc =
        ConnectionConfirm::new(0x1234, 0).with_negotiation(Protocol::Ssl as u32);

    let mut x224_cc_buffer = Vec::new();
    x224_cc.encode(&mut x224_cc_buffer).unwrap();

    let tpkt_cc = TpktPacket::new(x224_cc_buffer);
    let mut tpkt_cc_buffer = Vec::new();
    tpkt_cc.encode(&mut tpkt_cc_buffer).unwrap();

    let mut cursor = Cursor::new(&tpkt_cc_buffer);
    let decoded_tpkt_cc = TpktPacket::decode(&mut cursor).unwrap();

    let mut payload_cursor = Cursor::new(decoded_tpkt_cc.payload());
    let decoded_x224_cc = ConnectionConfirm::decode(&mut payload_cursor).unwrap();
    assert_eq!(
        decoded_x224_cc
            .rdp_negotiation()
            .unwrap()
            .selected_protocol,
        Protocol::Ssl as u32
    );

    // 3. MCS Connect-Initial (TPKT + X.224 Data)
    let gcc_data = b"GCC Conference Create Request data".to_vec();
    let mcs_ci = ConnectInitial::new(gcc_data.clone());

    let mut mcs_ci_buffer = Vec::new();
    mcs_ci.encode(&mut mcs_ci_buffer).unwrap();

    let x224_data = DataPdu::new(mcs_ci_buffer.clone());
    let mut x224_data_buffer = Vec::new();
    x224_data.encode(&mut x224_data_buffer).unwrap();

    let tpkt_data = TpktPacket::new(x224_data_buffer);
    let mut tpkt_data_buffer = Vec::new();
    tpkt_data.encode(&mut tpkt_data_buffer).unwrap();

    // 디코딩 검증
    let mut cursor = Cursor::new(&tpkt_data_buffer);
    let decoded_tpkt = TpktPacket::decode(&mut cursor).unwrap();

    let mut payload_cursor = Cursor::new(decoded_tpkt.payload());
    let decoded_x224_data = DataPdu::decode(&mut payload_cursor).unwrap();

    let mut mcs_cursor = Cursor::new(decoded_x224_data.payload());
    let decoded_mcs_ci = ConnectInitial::decode(&mut mcs_cursor).unwrap();
    assert_eq!(decoded_mcs_ci.user_data, gcc_data);

    // 4. MCS Connect-Response
    let response_data = b"GCC Conference Create Response data".to_vec();
    let mcs_cr = ConnectResponse::success(response_data.clone());

    let mut mcs_cr_buffer = Vec::new();
    mcs_cr.encode(&mut mcs_cr_buffer).unwrap();

    let mut cursor = Cursor::new(&mcs_cr_buffer);
    let decoded_mcs_cr = ConnectResponse::decode(&mut cursor).unwrap();
    assert_eq!(decoded_mcs_cr.result, McsResult::RtSuccessful);
    assert_eq!(decoded_mcs_cr.user_data, response_data);

    // 5. MCS Erect Domain Request
    let erect_domain = ErectDomainRequest::default_request();
    let mut erect_buffer = Vec::new();
    erect_domain.encode(&mut erect_buffer).unwrap();

    let mut cursor = Cursor::new(&erect_buffer);
    let decoded_erect = ErectDomainRequest::decode(&mut cursor).unwrap();
    assert_eq!(decoded_erect.sub_height, 0);
    assert_eq!(decoded_erect.sub_interval, 0);

    // 6. MCS Attach User Request/Confirm
    let attach_req = AttachUserRequest::new();
    let mut attach_req_buffer = Vec::new();
    attach_req.encode(&mut attach_req_buffer).unwrap();

    let mut cursor = Cursor::new(&attach_req_buffer);
    let _decoded_attach_req = AttachUserRequest::decode(&mut cursor).unwrap();

    let attach_confirm = AttachUserConfirm::success(1001);
    let mut attach_confirm_buffer = Vec::new();
    attach_confirm.encode(&mut attach_confirm_buffer).unwrap();

    let mut cursor = Cursor::new(&attach_confirm_buffer);
    let decoded_attach_confirm = AttachUserConfirm::decode(&mut cursor).unwrap();
    assert_eq!(decoded_attach_confirm.result, McsResult::RtSuccessful);
    assert_eq!(decoded_attach_confirm.user_id, Some(1001));

    // 7. MCS Channel Join Request/Confirm
    let channel_join_req = ChannelJoinRequest::new(1001, 1003);
    let mut channel_join_req_buffer = Vec::new();
    channel_join_req
        .encode(&mut channel_join_req_buffer)
        .unwrap();

    let mut cursor = Cursor::new(&channel_join_req_buffer);
    let decoded_channel_join_req = ChannelJoinRequest::decode(&mut cursor).unwrap();
    assert_eq!(decoded_channel_join_req.user_id, 1001);
    assert_eq!(decoded_channel_join_req.channel_id, 1003);

    let channel_join_confirm = ChannelJoinConfirm::success(1001, 1003);
    let mut channel_join_confirm_buffer = Vec::new();
    channel_join_confirm
        .encode(&mut channel_join_confirm_buffer)
        .unwrap();

    let mut cursor = Cursor::new(&channel_join_confirm_buffer);
    let decoded_channel_join_confirm = ChannelJoinConfirm::decode(&mut cursor).unwrap();
    assert_eq!(decoded_channel_join_confirm.result, McsResult::RtSuccessful);
    assert_eq!(decoded_channel_join_confirm.user_id, 1001);
    assert_eq!(decoded_channel_join_confirm.channel_id, Some(1003));
}

/// TPKT + X.224 + MCS 계층 통합 테스트
#[test]
fn test_layered_pdu_encoding() {
    // MCS ErectDomainRequest를 X.224 Data PDU로, 다시 TPKT으로 감싸기
    let mcs_pdu = ErectDomainRequest::new(1, 2);
    let mut mcs_buffer = Vec::new();
    mcs_pdu.encode(&mut mcs_buffer).unwrap();

    let x224_pdu = DataPdu::new(mcs_buffer);
    let mut x224_buffer = Vec::new();
    x224_pdu.encode(&mut x224_buffer).unwrap();

    let tpkt_pdu = TpktPacket::new(x224_buffer);
    let mut tpkt_buffer = Vec::new();
    tpkt_pdu.encode(&mut tpkt_buffer).unwrap();

    // 역방향 디코딩
    let mut cursor = Cursor::new(&tpkt_buffer);
    let decoded_tpkt = TpktPacket::decode(&mut cursor).unwrap();

    let mut cursor = Cursor::new(decoded_tpkt.payload());
    let decoded_x224 = DataPdu::decode(&mut cursor).unwrap();

    let mut cursor = Cursor::new(decoded_x224.payload());
    let decoded_mcs = ErectDomainRequest::decode(&mut cursor).unwrap();

    assert_eq!(decoded_mcs.sub_height, 1);
    assert_eq!(decoded_mcs.sub_interval, 2);
}

/// DomainParameters 기본값 테스트
#[test]
fn test_domain_parameters_defaults() {
    let target = DomainParameters::target();
    let minimum = DomainParameters::minimum();
    let maximum = DomainParameters::maximum();

    // Target은 reasonable한 값들
    assert_eq!(target.max_channel_ids, 34);
    assert_eq!(target.max_user_ids, 2);
    assert_eq!(target.max_mcspdu_size, 65535);

    // Minimum은 최소한의 값들
    assert_eq!(minimum.max_channel_ids, 1);
    assert_eq!(minimum.max_user_ids, 1);
    assert_eq!(minimum.max_mcspdu_size, 1056);

    // Maximum은 최대 값들
    assert_eq!(maximum.max_channel_ids, 65535);
    assert_eq!(maximum.max_user_ids, 64535);
    assert_eq!(maximum.max_mcspdu_size, 65535);
}
