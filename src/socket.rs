use crate::packet::TCPPacket;
use crate::tcpflags;
use anyhow::{Context, Result};
use pnet::packet::{ip::IpNextHeaderProtocols, Packet};
use pnet::transport::{self, TransportChannelType, TransportProtocol, TransportSender};
use pnet::util;
use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::net::{IpAddr, Ipv4Addr};
use std::time::SystemTime;

const SOCKET_BUFFER_SIZE: usize = 4380;

/// (local_addr, remote_addr, local_port, remote_port)
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct SockID(pub Ipv4Addr, pub Ipv4Addr, pub u16, pub u16);

#[derive(Clone, Debug)]
pub struct SendParam {
    pub unacked_seq: u32, // 送信後まだackされていないseqの先頭
    pub next: u32,        // 次の送信
    pub window: u16,      // 送信ウィンドウサイズ
    pub initial_seq: u32, // 初期送信seq
}

#[derive(Clone, Debug)]
pub struct RecvParam {
    pub next: u32,        // 次受信するseq
    pub window: u16,      // 受信ウィンドウ
    pub initial_seq: u32, // 初期受信seq
    pub tail: u32,        // 受信seqの最後尾
}

pub struct Socket {
    pub local_addr: Ipv4Addr,
    pub remote_addr: Ipv4Addr,
    pub local_port: u16,
    pub remote_port: u16,
    pub send_param: SendParam,
    pub recv_param: RecvParam,
    pub status: TcpStatus,
    pub recv_buffer: Vec<u8>,
    pub retransmission_queue: VecDeque<RetransmissionQueueEntry>,
    pub connected_connection_queue: VecDeque<SockID>, // 接続済みソケットを保持するキュー．リスニングソケットのみ使用．
    pub listening_socket: Option<SockID>, // 生成元のリスニングソケット．接続済みソケットのみ使用
    pub sender: TransportSender,
}

pub enum TcpStatus {}

impl Socket {
    pub fn new(
        local_addr: Ipv4Addr,
        remote_addr: Ipv4Addr,
        local_port: u16,
        remote_port: u16,
        status: TcpStatus,
    ) -> Result<Self> {
        let (sender, _) = transport::transport_channel(
            65535,
            TransportChannelType::Layer4(TransportProtocol::Ipv4(IpNextHeaderProtocols::Tcp)),
        )?;
        Ok(Self {
            local_addr,
            remote_addr,
            local_port,
            remote_port,
            send_param: SendParam {
                unacked_seq: 0,
                initial_seq: 0,
                next: 0,
                window: SOCKET_BUFFER_SIZE as u16,
            },
            recv_param: RecvParam {
                initial_seq: 0,
                next: 0,
                window: SOCKET_BUFFER_SIZE as u16,
                tail: 0,
            },
            status,
            recv_buffer: vec![0; SOCKET_BUFFER_SIZE],
            retransmission_queue: VecDeque::new(),
            connected_connection_queue: VecDeque::new(),
            listening_socket: None,
            sender,
        })
    }

    pub fn send_tcp_packet(&mut self, flag: u8, payload: &[u8]) -> Result<usize> {
        let mut tcp_packet = TCPPacket::new(payload.len());
        tcp_packet.set_src(self.local_port);
        tcp_packet.set_dest(self.remote_port);
        tcp_packet.set_flag(flag);

        let sent_size = self
            .sender
            .send_to(tcp_packet.clone(), IpAddr::V4(self.remote_addr))
            .unwrap();
        Ok(sent_size)
    }

    pub fn get_sock_id(&self) -> SockID {
        SockID(
            self.local_addr,
            self.remote_addr,
            self.local_port,
            self.remote_port,
        )
    }
}
