# RDP PDU 모듈 설계 문서

## 1. 전체 모듈 계층 구조

RDP 프로토콜은 여러 계층으로 구성되므로, 다음과 같은 모듈 구조를 제안합니다:

```
src/
├── lib.rs
├── pdu/
│   ├── mod.rs                  # PDU 공통 trait 및 유틸리티 (선언 전용)
│   ├── tpkt.rs                 # TPKT Layer (단일 파일 모듈)
│   ├── x224/                   # X.224 Connection-Oriented Transport (디렉토리 모듈)
│   │   ├── mod.rs              # mod connection; mod data; mod disconnect; 선언
│   │   ├── connection.rs       # Connection Request/Confirm
│   │   ├── data.rs             # Data Transfer
│   │   └── disconnect.rs       # Disconnect Request
│   ├── mcs/                    # MCS Layer (디렉토리 모듈)
│   │   ├── mod.rs              # mod connect; mod channel; mod domain; 선언
│   │   ├── connect.rs          # MCS Connect Initial/Response
│   │   ├── channel.rs          # Channel Join/Attach
│   │   └── domain.rs           # Domain PDUs
│   └── rdp/                    # RDP Core Layer (디렉토리 모듈)
│       ├── mod.rs              # RDP 공통, mod capability; mod connection; mod input; mod graphics; 선언
│       ├── capability/         # Capability Sets (디렉토리 모듈)
│       │   ├── mod.rs          # mod general; mod bitmap; mod order; mod input; 선언
│       │   ├── general.rs
│       │   ├── bitmap.rs
│       │   ├── order.rs
│       │   └── input.rs
│       ├── connection/         # Connection Sequence (디렉토리 모듈)
│       │   ├── mod.rs          # mod client_info; mod server_demand; 선언
│       │   ├── client_info.rs
│       │   └── server_demand.rs
│       ├── input/              # Input PDUs (디렉토리 모듈)
│       │   ├── mod.rs          # mod keyboard; mod mouse; 선언
│       │   ├── keyboard.rs
│       │   └── mouse.rs
│       └── graphics/           # Graphics Update PDUs (디렉토리 모듈)
│           ├── mod.rs          # mod bitmap; mod orders; 선언
│           ├── bitmap.rs
│           └── orders.rs
```

## 2. 핵심 설계 원칙

### A. 공통 Trait 정의 (`pdu/mod.rs`)

```rust
// PDU 공통 인터페이스
pub trait Pdu: Sized {
    fn encode(&self, buffer: &mut impl Write) -> Result<()>;
    fn decode(buffer: &mut impl Read) -> Result<Self>;
    fn size(&self) -> usize;
}

// 헤더가 있는 PDU
pub trait PduWithHeader: Pdu {
    type Header;
    fn header(&self) -> &Self::Header;
}
```

### B. 바이트 순서 및 인코딩

- Little-endian (RDP 표준)
- BER (Basic Encoding Rules) for MCS
- 의존성: `byteorder`, `bytes` 크레이트

### C. 에러 처리

```rust
pub enum PduError {
    InvalidLength,
    InvalidHeader,
    UnsupportedVersion,
    ParseError(String),
    IoError(std::io::Error),
}
```

## 3. 각 계층별 상세 설계

### TPKT Layer (`pdu/tpkt/`)

#### 패킷 구조
```
[TPKT Header - 4 bytes]
- Version (1 byte): 0x03
- Reserved (1 byte): 0x00
- Length (2 bytes): 전체 패킷 길이 (빅엔디안)
- Payload: X.224 data
```

#### 주요 구조체
- `TpktHeader`
- `TpktPacket`

### X.224 Layer (`pdu/x224/`)

#### 패킷 구조
```
[X.224 Header - 가변 길이]
- Length Indicator (1 byte): 헤더 길이
- PDU Type (1 byte): CR, CC, DT, DR 등
- 추가 필드 (PDU 타입에 따라 다름)
```

#### 주요 타입
- `ConnectionRequest` (CR) - 0xE0
- `ConnectionConfirm` (CC) - 0xD0
- `Data` (DT) - 0xF0
- `DisconnectRequest` (DR) - 0x80

### MCS Layer (`pdu/mcs/`)

BER 인코딩 사용:

#### 주요 PDU
- `ConnectInitial` - GCC Conference Create Request 포함
- `ConnectResponse` - GCC Conference Create Response 포함
- `ErectDomainRequest`
- `AttachUserRequest/Confirm`
- `ChannelJoinRequest/Confirm`
- `SendDataRequest/Indication`

#### 특징
- PER (Packed Encoding Rules) 사용
- 복잡한 ASN.1 구조
- Choice 타입 인코딩

### RDP Core Layer (`pdu/rdp/`)

#### 연결 시퀀스 PDU
1. `ClientInfoPdu` - 사용자 인증 정보
2. `ServerLicenseErrorPdu`
3. `ClientConfirmActivePdu` - Capability Sets 포함
4. `ServerDemandActivePdu`
5. `SynchronizePdu`
6. `ControlPdu`
7. `FontListPdu`

#### Capability Sets
- General Capability
- Bitmap Capability
- Order Capability
- Input Capability
- Brush Capability
- Glyph Capability
- Pointer Capability
- Sound Capability
- Font Capability
- 등등 (약 20+ 종류)

## 4. 유틸리티 모듈

```
src/
├── io/                        # 입출력 헬퍼
│   ├── mod.rs
│   ├── cursor.rs             # 커서 기반 읽기/쓰기
│   └── buffer.rs             # 버퍼 관리
└── codec/                     # 인코딩/디코딩
    ├── mod.rs
    ├── ber.rs                # BER 인코더/디코더
    └── per.rs                # PER 인코더/디코더
```

## 5. 의존성 제안 (`Cargo.toml`)

```toml
[dependencies]
bytes = "1.5"              # 효율적인 바이트 버퍼
byteorder = "1.5"          # 바이트 순서 변환
thiserror = "1.0"          # 에러 정의
num_enum = "0.7"           # enum ↔ 숫자 변환
bitflags = "2.4"           # 플래그 비트 처리

[dev-dependencies]
hex = "0.4"                # 테스트용 hex 인코딩
```

## 6. 구현 우선순위

### Phase 1 - 기본 계층 ✅ 완료
- [x] 공통 trait 및 에러 타입 (`pdu/mod.rs`)
- [x] TPKT 패킷 (`pdu/tpkt.rs`)
- [x] X.224 Data 패킷 (`pdu/x224/data.rs`)
- [x] 단위 테스트 (16개 테스트 통과)

### Phase 2 - 연결 설정
- [ ] X.224 Connection Request/Confirm
- [ ] MCS Connect Initial/Response
- [ ] Channel Join 시퀀스

### Phase 3 - RDP 코어
- [ ] Client Info PDU
- [ ] Capability Sets
- [ ] Control/Synchronize PDUs

### Phase 4 - 데이터 전송
- [ ] Input PDUs (키보드, 마우스)
- [ ] Graphics Update PDUs

## 7. 테스트 전략

### 단위 테스트
- 각 PDU별 encode/decode 테스트
- 경계값 테스트
- 에러 케이스 테스트

### 통합 테스트
- 실제 RDP 트래픽 캡처 데이터로 테스트
- Wireshark pcap 파일 파싱 테스트

### 속성 기반 테스트
- encode → decode → 원본 일치 검증
- 랜덤 데이터 생성 후 roundtrip 테스트

## 8. 참고 자료

### 프로토콜 스펙
- [MS-RDPBCGR]: Remote Desktop Protocol: Basic Connectivity and Graphics Remoting
- [MS-RDPEGDI]: Remote Desktop Protocol: Graphics Device Interface (GDI) Acceleration Extensions
- RFC 1006: ISO transport services on top of the TCP
- ISO 8073: X.224 Connection-Oriented Transport Protocol
- T.125: Multipoint Communication Service Protocol

### 기존 구현체
- FreeRDP (C/C++)
- Remmina (C)
- xrdp (C)

## 9. 설계 장점

- ✅ 계층적이고 명확한 구조
- ✅ 재사용 가능한 trait 기반 설계
- ✅ 타입 안전성 보장
- ✅ 확장 가능한 아키텍처
- ✅ 테스트 가능한 모듈 분리

## 10. 향후 고려사항

### 성능 최적화
- Zero-copy 파싱 (bytes 크레이트 활용)
- 버퍼 풀링
- SIMD 활용 (비트맵 디코딩 등)

### 보안
- 입력 검증 강화
- 메모리 안전성 (Rust의 장점)
- TLS/SSL 지원 (rustls)

### 확장성
- RDP 8.0+ 기능 지원
- RemoteFX
- 멀티모니터 지원
- 오디오/비디오 리다이렉션
