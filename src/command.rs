// Command definitions
// Overall structure is modeled after https://github.com/tokio-rs/mini-redis/blob/master/src/cmd/mod.rs
use super::error::{Error, ErrorKind};
use super::memory::{Size, Slot, Zone};
use super::packet::{Packet, PacketBuilder};
use core::convert::TryFrom;

/// AES mode: Encrypt
const MODE_ENCRYPT: u8 = 0x00;
/// AES mode: Decrypt
const MODE_DECRYPT: u8 = 0x01;
/// Initialization, does not accept a message
const MODE_SHA256_START: u8 = 0x00;
/// Add 64 bytes in the meesage to the SHA context
const MODE_SHA256_UPDATE: u8 = 0x01;
/// Complete the calculation and return the digest
const MODE_SHA256_END: u8 = 0x02;
/// Add 64 byte ECC public key in the slot to the SHA context
#[allow(dead_code)]
const MODE_SHA256_PUBLIC: u8 = 0x03;
/// Info mode Revision
const MODE_REVISION: u8 = 0x00;

// Enumerate objects you may want from the device. Provide a bunch of
// specialized return types since most of the commands return status code only.

/// Revision number and so on.
/// A return type of API `info`.
#[derive(Clone, Copy, Debug)]
pub struct Word {
    value: [u8; 0x04],
}

// Parse a serial number from response buffer.
impl TryFrom<&[u8]> for Word {
    type Error = Error;
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() != Size::Word as usize {
            return Err(ErrorKind::BadParam.into());
        }
        let mut value = [0; 0x04];
        value.as_mut().copy_from_slice(&buffer[0..4]);
        Ok(Self { value })
    }
}

impl AsRef<[u8]> for Word {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

/// What's this?
/// A return type of which API?
#[derive(Clone, Copy, Debug)]
pub struct Block {
    value: [u8; 0x10],
}

impl AsRef<[u8]> for Block {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

/// Serial number of 9 bytes. Its uniqueness is guaranteed.
/// A return type of API `read_serial`.
#[derive(Clone, Copy, Debug)]
pub struct Serial {
    value: [u8; 9],
}

// Parse a serial number from response buffer.
impl TryFrom<&[u8]> for Serial {
    type Error = Error;
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() >= 9 {
            return Err(ErrorKind::BadParam.into());
        }
        let mut value = [0; 9];
        value.as_mut().copy_from_slice(&buffer[0..4]);
        value.as_mut().copy_from_slice(&buffer[8..13]);
        Ok(Serial { value })
    }
}

impl AsRef<[u8]> for Serial {
    fn as_ref(&self) -> &[u8] {
        &self.value
    }
}

/// A public key signature returned from a signing operation. Format is R and
/// S integers in big-endian format. 64 bytes for P256 curve.
#[derive(Clone, Copy, Debug)]
pub struct Signature {
    value: [u8; 64],
}

// A digest yielded from cryptographic hash functions.
// For reference, `digest` crate uses `GenericArray<u8, 32>`.
#[derive(Clone, Copy, Debug)]
pub struct Digest {
    value: [u8; 32],
}

// Parse digest from response buffer.
impl TryFrom<&[u8]> for Digest {
    type Error = Error;
    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() != 32 {
            return Err(ErrorKind::BadParam.into());
        }
        let mut value = [0; 32];
        value.as_mut().copy_from_slice(buffer.as_ref());
        Ok(Self { value })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PremasterSecret {
    value: [u8; 32],
}

#[derive(Clone, Copy, Debug)]
pub struct Nonce {
    value: [u8; 32],
}

#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    /// CheckMac command op-code
    CheckMac = 0x28,
    /// DeriveKey command op-code
    DeriveKey = 0x1C,
    /// Info command op-code
    Info = 0x30,
    /// GenDig command op-code
    GenDig = 0x15,
    /// GenKey command op-code
    GenKey = 0x40,
    /// HMAC command op-code
    HMac = 0x11,
    /// Lock command op-code
    Lock = 0x17,
    /// MAC command op-code
    Mac = 0x08,
    /// Nonce command op-code
    Nonce = 0x16,
    /// Pause command op-code
    Pause = 0x01,
    /// PrivWrite command op-code
    PrivWrite = 0x46,
    /// Random command op-code
    Random = 0x1B,
    /// Read command op-code
    Read = 0x02,
    /// Sign command op-code
    Sign = 0x41,
    /// UpdateExtra command op-code
    UpdateExtra = 0x20,
    /// GenKey command op-code
    Verify = 0x45,
    /// Write command op-code
    Write = 0x12,
    /// ECDH command op-code
    Ecdh = 0x43,
    /// Counter command op-code
    Counter = 0x24,
    /// SHA command op-code
    Sha = 0x47,
    /// AES command op-code
    Aes = 0x51,
    /// KDF command op-code
    Kdf = 0x56,
    /// Secure Boot command op-code
    SecureBoot = 0x80,
    /// Self test command op-code
    SelfTest = 0x77,
}

/// ChipMode clock divider {M0, M1, M2}
#[derive(Clone, Copy, Debug)]
pub enum ClockDivider {
    Zero = 0,
    One = 1,
    Two = 2,
}

const EXEC_TIME_AES: [u32; 3] = [27, 27, 27];
const EXEC_TIME_CHECKMAC: [u32; 3] = [40, 40, 40];
const EXEC_TIME_COUNTER: [u32; 3] = [25, 25, 25];
const EXEC_TIME_DERIVE_KEY: [u32; 3] = [50, 50, 50];
const EXEC_TIME_ECDH: [u32; 3] = [172, 75, 531];
const EXEC_TIME_GENDIG: [u32; 3] = [35, 25, 35];
const EXEC_TIME_GENKEY: [u32; 3] = [215, 115, 653];
const EXEC_TIME_INFO: [u32; 3] = [5, 5, 5];
const EXEC_TIME_KDF: [u32; 3] = [165, 165, 165];
const EXEC_TIME_LOCK: [u32; 3] = [35, 35, 35];
const EXEC_TIME_MAC: [u32; 3] = [55, 55, 55];
const EXEC_TIME_NONCE: [u32; 3] = [20, 20, 20];
const EXEC_TIME_PRIVWRITE: [u32; 3] = [50, 50, 50];
const EXEC_TIME_RANDOM: [u32; 3] = [23, 23, 23];
const EXEC_TIME_READ: [u32; 3] = [5, 5, 5];
const EXEC_TIME_SECUREBOOT: [u32; 3] = [160, 80, 480];
const EXEC_TIME_SELFTEST: [u32; 3] = [625, 250, 2324];
const EXEC_TIME_SHA: [u32; 3] = [36, 42, 75];
const EXEC_TIME_SIGN: [u32; 3] = [115, 220, 665];
const EXEC_TIME_UPDATE_EXTRA: [u32; 3] = [10, 10, 10];
const EXEC_TIME_VERIFY: [u32; 3] = [105, 295, 1085];
const EXEC_TIME_WRITE: [u32; 3] = [45, 45, 45];

impl OpCode {
    /// Get the typical execution time for the given command.
    pub fn execution_time(&self, div: ClockDivider) -> Option<u32> {
        use OpCode::*;
        match self {
            CheckMac => EXEC_TIME_CHECKMAC[div as usize].into(),
            Counter => EXEC_TIME_COUNTER[div as usize].into(),
            DeriveKey => EXEC_TIME_DERIVE_KEY[div as usize].into(),
            Ecdh => EXEC_TIME_ECDH[div as usize].into(),
            GenDig => EXEC_TIME_GENDIG[div as usize].into(),
            GenKey => EXEC_TIME_GENKEY[div as usize].into(),
            Info => EXEC_TIME_INFO[div as usize].into(),
            Lock => EXEC_TIME_LOCK[div as usize].into(),
            Mac => EXEC_TIME_MAC[div as usize].into(),
            Nonce => EXEC_TIME_NONCE[div as usize].into(),
            PrivWrite => EXEC_TIME_PRIVWRITE[div as usize].into(),
            Random => EXEC_TIME_RANDOM[div as usize].into(),
            Read => EXEC_TIME_READ[div as usize].into(),
            Sign => EXEC_TIME_SIGN[div as usize].into(),
            UpdateExtra => EXEC_TIME_UPDATE_EXTRA[div as usize].into(),
            Verify => EXEC_TIME_VERIFY[div as usize].into(),
            Write => EXEC_TIME_WRITE[div as usize].into(),
            Sha => EXEC_TIME_SHA[div as usize].into(),
            Aes => EXEC_TIME_AES[div as usize].into(),
            Kdf => EXEC_TIME_KDF[div as usize].into(),
            SecureBoot => EXEC_TIME_SECUREBOOT[div as usize].into(),
            SelfTest => EXEC_TIME_SELFTEST[div as usize].into(),
            _ => None,
        }
    }
}

pub(crate) struct CheckMac<'a>(PacketBuilder<'a>);
pub(crate) struct Counter<'a>(PacketBuilder<'a>);
pub(crate) struct DeriveKey<'a>(PacketBuilder<'a>);
pub(crate) struct Ecdh<'a>(PacketBuilder<'a>);
/// Generate Digest
pub(crate) struct GenDig<'a>(PacketBuilder<'a>);
pub(crate) struct GenKey<'a>(PacketBuilder<'a>);
pub(crate) struct HMac<'a>(PacketBuilder<'a>);
pub(crate) struct Info<'a>(PacketBuilder<'a>);
pub(crate) struct Lock<'a>(PacketBuilder<'a>);
pub(crate) struct Mac<'a>(PacketBuilder<'a>);
pub(crate) struct NonceCmd<'a>(PacketBuilder<'a>);
pub(crate) struct Pause<'a>(PacketBuilder<'a>);

// For best security, it is recommended that the `PrivWrite` command not be used,
// and that private keys be internally generated from the RNG using the `GenKey`
// command.
#[allow(dead_code)]
pub(crate) struct PrivWrite<'a>(PacketBuilder<'a>);
pub(crate) struct Random<'a>(PacketBuilder<'a>);
pub(crate) struct Read<'a>(PacketBuilder<'a>);
pub(crate) struct Sign<'a>(PacketBuilder<'a>);
pub(crate) struct UpdateExtra<'a>(PacketBuilder<'a>);
pub(crate) struct Verify<'a>(PacketBuilder<'a>);
pub(crate) struct Write<'a>(PacketBuilder<'a>);
pub(crate) struct Sha<'a>(PacketBuilder<'a>);
pub(crate) struct Aes<'a>(PacketBuilder<'a>);
pub(crate) struct Kdf<'a>(PacketBuilder<'a>);
pub(crate) struct SecureBoot<'a>(PacketBuilder<'a>);
pub(crate) struct SelfTest<'a>(PacketBuilder<'a>);

// Used when signing an internally stored digest. The GenDig command uses
// SHA-256 to combine a stored value with the contents of TempKey, which must
// have been valid prior to the execution of this command.
#[allow(dead_code)]
impl<'a> GenDig<'a> {
    pub(crate) fn new(builder: PacketBuilder<'a>) -> Self {
        Self(builder)
    }

    pub(crate) fn gendig(&mut self, key_id: Slot) -> Result<Packet, Error> {
        let packet = self.0.opcode(OpCode::GenDig).param2(key_id as u16).build();
        Ok(packet)
    }
}

impl<'a> Info<'a> {
    pub(crate) fn new(builder: PacketBuilder<'a>) -> Self {
        Self(builder)
    }

    /// Command execution will return a word containing the revision.
    pub(crate) fn revision(&mut self) -> Result<Packet, Error> {
        let packet = self.0.opcode(OpCode::Info).mode(MODE_REVISION).build();
        Ok(packet)
    }
}

/// Nonce
impl<'a> NonceCmd<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(builder: PacketBuilder<'a>) -> Self {
        Self(builder)
    }

    // TODO: Usage of Nonce is not clear. In `test/api_atcab/atca_tests_aes.c`, AES
    // encryption/decryption assumes Nonce value is loaded to TempKey in advance.
    /*
        // Load AES keys into TempKey
        pub(crate) fn load(&mut self) -> Result<Packet, Error> {
            nonce_load(NONCE_MODE_TARGET_TEMPKEY, g_aes_keys[0], 64);
        }
    */
}

impl<'a> Sha<'a> {
    pub(crate) fn new(builder: PacketBuilder<'a>) -> Self {
        Self(builder)
    }

    pub(crate) fn start(&mut self) -> Result<Packet, Error> {
        let packet = self.0.opcode(OpCode::Sha).mode(MODE_SHA256_START).build();
        Ok(packet)
    }

    /// Data length cannot exceed 64 bytes.
    pub(crate) fn update(&mut self, data: impl AsRef<[u8]>) -> Result<Packet, Error> {
        if data.as_ref().len() >= 64 {
            return Err(ErrorKind::BadParam.into());
        }

        let packet = self
            .0
            .opcode(OpCode::Sha)
            .mode(MODE_SHA256_UPDATE)
            .pdu_data(data)
            .build();
        Ok(packet)
    }

    /// Command execution will return a digest of Block size.
    pub(crate) fn end(&mut self) -> Result<Packet, Error> {
        let packet = self.0.opcode(OpCode::Sha).mode(MODE_SHA256_END).build();
        Ok(packet)
    }
}

/// AES
impl<'a> Aes<'a> {
    pub(crate) fn new(builder: PacketBuilder<'a>) -> Self {
        Self(builder)
    }

    /// Plain text has length of 16 bytes.
    pub(crate) fn encrypt(&mut self, slot: Slot, plaintext: &[u8]) -> Result<Packet, Error> {
        if !slot.is_private_key() {
            return Err(ErrorKind::BadParam.into());
        }

        // Input length should be exactly 16 bytes. Otherwise the device
        // couldn't recognize the command properly.
        if plaintext.len() > 16 as usize {
            return Err(ErrorKind::InvalidSize.into());
        }

        let packet = self
            .0
            .opcode(OpCode::Aes)
            .mode(MODE_ENCRYPT)
            .param2(slot as u16)
            .pdu_data(plaintext)
            .pdu_length(16)
            .build();
        Ok(packet)
    }

    /// Cipher text has length of 16 bytes.
    pub(crate) fn decrypt(&mut self, slot: Slot, ciphertext: &[u8]) -> Result<Packet, Error> {
        if !slot.is_private_key() {
            return Err(ErrorKind::BadParam.into());
        }

        // Input length should be exactly 16 bytes. Otherwise the device
        // couldn't recognize the command properly.
        if ciphertext.len() != 16 as usize {
            return Err(ErrorKind::InvalidSize.into());
        }

        let packet = self
            .0
            .opcode(OpCode::Aes)
            .mode(MODE_DECRYPT)
            .param2(slot as u16)
            .pdu_data(ciphertext)
            .pdu_length(16)
            .build();
        Ok(packet)
    }
}

/// Read
impl<'a> Read<'a> {
    pub(crate) fn new(builder: PacketBuilder<'a>) -> Self {
        Self(builder)
    }

    pub(crate) fn slot(&mut self, slot: Slot, block: u8) -> Result<Packet, Error> {
        let addr = Zone::Data.get_slot_addr(slot, block)?;
        let mode = Zone::Data.encode(Size::Block);
        let packet = self.0.opcode(OpCode::Read).mode(mode).param2(addr).build();
        Ok(packet)
    }

    pub(crate) fn register(
        &mut self,
        zone: Zone,
        size: Size,
        block: u8,
        offset: u8,
    ) -> Result<Packet, Error> {
        let addr = zone.get_addr(block, offset)?;
        let mode = zone.encode(size);
        let packet = self.0.opcode(OpCode::Read).mode(mode).param2(addr).build();
        Ok(packet)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha() {
        let buf = &mut [0x00u8; 0xff];
        let packet = Sha::new(PacketBuilder::new(buf.as_mut()))
            .start()
            .unwrap()
            .buffer(buf.as_ref());
        assert_eq!(packet[0x01], 0x07);
        assert_eq!(packet[0x02], OpCode::Sha as u8);
        assert_eq!(packet[0x03], MODE_SHA256_START);
        assert_eq!(packet[0x04..0x06], [0x00, 0x00]);
    }
}
