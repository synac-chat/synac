extern crate rmp_serde as rmps;
#[macro_use] extern crate serde_derive;

use std::collections::HashMap;
use std::io;

pub const DEFAULT_PORT: u16  = 8439;
pub const TYPING_TIMEOUT: u8 = 10;

pub const LIMIT_USER_NAME:    usize = 128;
pub const LIMIT_CHANNEL_NAME: usize = 128;
pub const LIMIT_ATTR_NAME:    usize = 128;
pub const LIMIT_ATTR_AMOUNT:  usize = 2048;
pub const LIMIT_MESSAGE:      usize = 16384;

pub const LIMIT_MESSAGE_LIST: usize = 64;

pub const ERR_ATTR_INVALID_POS:   u8 = 1;
pub const ERR_ATTR_LOCKED_NAME:   u8 = 2;
pub const ERR_LIMIT_REACHED:      u8 = 3;
pub const ERR_LOGIN_BANNED:       u8 = 4;
pub const ERR_LOGIN_BOT:          u8 = 5;
pub const ERR_LOGIN_INVALID:      u8 = 6;
pub const ERR_MAX_CONN_PER_IP:    u8 = 7;
pub const ERR_MISSING_FIELD:      u8 = 8;
pub const ERR_MISSING_PERMISSION: u8 = 9;
pub const ERR_UNKNOWN_ATTRIBUTE:  u8 = 10;
pub const ERR_UNKNOWN_CHANNEL:    u8 = 11;
pub const ERR_UNKNOWN_MESSAGE:    u8 = 12;
pub const ERR_UNKNOWN_USER:       u8 = 13;

pub const PERM_READ:              u8 = 1;
pub const PERM_WRITE:             u8 = 1 << 1;

pub const PERM_ASSIGN_ATTRIBUTES: u8 = 1 << 2;
pub const PERM_BAN:               u8 = 1 << 3;
pub const PERM_MANAGE_ATTRIBUTES: u8 = 1 << 4;
pub const PERM_MANAGE_CHANNELS:   u8 = 1 << 5;
pub const PERM_MANAGE_MESSAGES:   u8 = 1 << 6;

// TYPES
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Attribute {
    pub allow: u8,
    pub deny: u8,
    pub id: usize,
    pub name: String,
    pub pos: usize,
    pub unassignable: bool
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct User {
    pub attributes: Vec<usize>,
    pub ban: bool,
    pub bot: bool,
    pub id: usize,
    pub name: String
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Channel {
    pub id: usize,
    pub name: String,
    pub overrides: HashMap<usize, (u8, u8)>
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    pub author: usize,
    pub channel: usize,
    pub id: usize,
    pub text: Vec<u8>,
    pub timestamp: i64,
    pub timestamp_edit: Option<i64>
}

// CLIENT PACKETS
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Close {}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Login {
    pub bot: bool,
    pub name: String,
    pub password: Option<String>,
    pub token: Option<String>
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AttributeCreate {
    pub allow: u8,
    pub deny: u8,
    pub name: String,
    pub pos: usize,
    pub unassignable: bool
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AttributeUpdate {
    pub inner: Attribute
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AttributeDelete {
    pub id: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChannelCreate {
    pub name: String,
    pub overrides: HashMap<usize, (u8, u8)>
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChannelUpdate {
    pub inner: Channel,
    pub keep_overrides: bool
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChannelDelete {
    pub id: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageList {
    pub after: Option<usize>,
    pub before: Option<usize>,
    pub channel: usize,
    pub limit: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Typing {
    pub channel: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageCreate {
    pub channel: usize,
    pub text: Vec<u8>
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageUpdate {
    pub id: usize,
    pub text: Vec<u8>
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageDelete {
    pub id: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Command {
    pub author: usize,
    pub parts: Vec<String>,
    pub recipient: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserUpdate {
    pub attributes: Option<Vec<usize>>,
    pub ban: Option<bool>,
    pub id: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LoginUpdate {
    pub name: Option<String>,
    pub password_current: Option<String>,
    pub password_new: Option<String>,
    pub reset_token: bool
}

// SERVER PACKETS
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LoginSuccess {
    pub created: bool,
    pub id: usize,
    pub token: String
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserReceive {
    pub inner: User
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AttributeReceive {
    pub inner: Attribute,
    pub new: bool
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AttributeDeleteReceive {
    pub inner: Attribute
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChannelReceive {
    pub inner: Channel
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChannelDeleteReceive {
    pub inner: Channel
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TypingReceive {
    pub author: usize,
    pub channel: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageReceive {
    pub author: User,
    pub channel: Channel,
    pub id: usize,
    pub new: bool,
    pub text: Vec<u8>,
    pub timestamp: i64,
    pub timestamp_edit: Option<i64>
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MessageDeleteReceive {
    pub id: usize
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CommandReceive {
    pub args: Vec<String>,
    pub author: User,
    pub command: String,
    pub recipient: User
}

macro_rules! packet {
    ($($type:ident),+) => {
        #[derive(Serialize, Deserialize, Debug)]
        #[serde(/*tag = "type",*/ rename_all = "snake_case")]
        pub enum Packet {
            Close,
            Err(u8),
            RateLimited(u64),
            $($type($type),)+
        }
    }
}
packet! {
    Login,
    AttributeCreate,
    AttributeUpdate,
    AttributeDelete,
    ChannelCreate,
    ChannelUpdate,
    ChannelDelete,
    MessageList,
    Typing,
    MessageCreate,
    MessageUpdate,
    MessageDelete,
    Command,
    UserUpdate,
    LoginUpdate,

    LoginSuccess,
    UserReceive,
    AttributeReceive,
    AttributeDeleteReceive,
    ChannelReceive,
    ChannelDeleteReceive,
    TypingReceive,
    MessageReceive,
    MessageDeleteReceive,
    CommandReceive
}

pub fn serialize(packet: &Packet) -> Result<Vec<u8>, rmps::encode::Error> {
    rmps::to_vec(&packet)
}
pub fn deserialize(buf: &[u8]) -> Result<Packet, rmps::decode::Error> {
    rmps::from_slice(buf)
}
pub fn deserialize_stream<T: io::Read>(buf: T) -> Result<Packet, rmps::decode::Error> {
    rmps::from_read(buf)
}
pub fn encode_u16(input: u16) -> [u8; 2] {
    [
        (input >> 8)  as u8,
        (input % 256) as u8
    ]
}
pub fn decode_u16(bytes: &[u8]) -> u16 {
    assert_eq!(bytes.len(), 2);

    ((bytes[0] as u16) << 8) + bytes[1] as u16
}

#[derive(Debug)]
pub enum Error {
    DecodeError(rmps::decode::Error),
    EncodeError(rmps::encode::Error),
    ErrPacketTooBig,
    IoError(std::io::Error)
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::DecodeError(ref inner) => inner.description(),
            Error::EncodeError(ref inner) => inner.description(),
            Error::ErrPacketTooBig    => "Packet size must fit into an u16",
            Error::IoError(ref inner)     => inner.description()
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::error::Error as StdError;
        match *self {
            Error::DecodeError(ref inner) => write!(f, "{}", inner),
            Error::EncodeError(ref inner) => write!(f, "{}", inner),
            Error::ErrPacketTooBig        => write!(f, "{}", self.description()),
            Error::IoError(ref inner)     => write!(f, "{}", inner)
        }
    }
}
impl From<rmps::decode::Error> for Error {
    fn from(err: rmps::decode::Error) -> Self {
        Error::DecodeError(err)
    }
}
impl From<rmps::encode::Error> for Error {
    fn from(err: rmps::encode::Error) -> Self {
        Error::EncodeError(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

pub fn read<T: io::Read>(reader: &mut T) -> Result<Packet, Error> {
    let mut buf = [0; 2];
    reader.read_exact(&mut buf)?;

    let size = decode_u16(&buf) as usize;
    let mut buf = vec![0; size];
    reader.read_exact(&mut buf)?;

    Ok(deserialize(&buf)?)
}
pub fn write<T: io::Write>(writer: &mut T, packet: &Packet) -> Result<(), Error> {
    let buf = serialize(packet)?;
    if buf.len() > std::u16::MAX as usize {
        return Err(Error::ErrPacketTooBig);
    }
    let size = encode_u16(buf.len() as u16);
    writer.write_all(&size)?;
    writer.write_all(&buf)?;
    writer.flush()?;

    Ok(())
}

pub fn perm_apply_iter<I: Iterator<Item = (u8, u8)>>(into: &mut u8, attributes: &mut I) {
    // Expects attributes to be sorted
    for attribute in attributes {
        perm_apply(into, attribute);
    }
}
pub fn perm_apply(into: &mut u8, (allow, deny): (u8, u8)) {
    *into |= allow;
    *into &= !deny;
}
