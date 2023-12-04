// Pre-configured device memory state and pre-provisioned key store access
// management. 1. Fully specified configuration zone. 2. One permanent primary
// P-256 Elliptic Curve Cryptography (ECC) private key fixed at the first object
// creation. 3. One internal sign private key for key attestation. 4. Three
// secondary P-256 ECC private keys that can be regenerated by the user. 5.
// Signer public key from signer certificate. 6. ECDH/KDF key slot capable of
// being used with AES keys and commands. 7. X.509 Compressed Certificate
// Storage.
use super::client::{AtCaClient, Memory, Sha};
use super::error::Error;
use super::memory::{Size, Slot, Zone};
use core::convert::TryFrom;
use digest::{FixedOutputDirty, Reset, Update};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c;
use generic_array::typenum::U32;
use generic_array::GenericArray;

pub const AUTH_PRIVATE_KEY: Slot = Slot::PrivateKey00;
pub const SIGN_PRIVATE_KEY: Slot = Slot::PrivateKey01;
pub const USER_PRIVATE_KEY1: Slot = Slot::PrivateKey02;
pub const USER_PRIVATE_KEY2: Slot = Slot::PrivateKey03;
pub const USER_PRIVATE_KEY3: Slot = Slot::PrivateKey04;
pub const IO_PROTECTION_KEY: Slot = Slot::PrivateKey06;
pub const AES_KEY: Slot = Slot::Certificate09;
pub const DEVICE_CERTIFICATE: Slot = Slot::Certificate0a;
pub const SIGNER_PUBLIC_KEY: Slot = Slot::Certificate0b;
pub const SIGNER_CERTIFICATE: Slot = Slot::Certificate0c;

pub struct Hasher<'a, PHY, D>(Sha<'a, PHY, D>);
impl<'a, PHY, D> From<Sha<'a, PHY, D>> for Hasher<'a, PHY, D> {
    fn from(sha: Sha<'a, PHY, D>) -> Self {
        Self(sha)
    }
}

impl<'a, PHY, D> Update for Hasher<'a, PHY, D>
where
    PHY: i2c::I2c,
    D: DelayNs,
{
    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.0.update(data).expect("update operation failed");
    }
}

impl<'a, PHY, D> FixedOutputDirty for Hasher<'a, PHY, D>
where
    PHY: i2c::I2c,
    D: DelayNs,
{
    type OutputSize = U32;
    fn finalize_into_dirty(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        let digest = self.0.finalize().expect("finalize operation failed");
        out.as_mut_slice().copy_from_slice(digest.as_ref());
    }
}

impl<'a, PHY, D> Reset for Hasher<'a, PHY, D>
where
    PHY: i2c::I2c,
    D: DelayNs,
{
    fn reset(&mut self) {}
}

pub struct TrustAndGo<'a, PHY, D> {
    atca: &'a mut AtCaClient<PHY, D>,
}

impl<'a, PHY, D> TrustAndGo<'a, PHY, D> {
    // Miscellaneous device states.
    const TNG_TLS_SLOT_CONFIG_DATA: [u8; Size::Block as usize] = [
        // Index 20..=51, block = 0, offset = 5
        0x85, 0x00, // Slot 0x00, Primary private key
        0x82, 0x00, // Slot 0x01, Internal sign private key
        0x85, 0x20, 0x85, 0x20, 0x85, 0x20, // Slot 02, 03 and 04, Secondary private keys 1-3
        0x8f, 0x8f, // Slot 0x05, reserved.
        0x8f, 0x0f, // Slot 0x06, I/O protection key
        0xaf, 0x8f, // Slot 0x07, reserved.
        0x0f, 0x0f, // Slot 0x08, General data
        0x8f, 0x0f, // Slot 0x09, AES key
        0x0f, 0x8f, // Slot 0x0a, Device compressed certificate
        0x0f, 0x8f, // Slot 0x0b, Signer public key
        0x0f, 0x8f, // Slot 0x0c, Signer compressed certificate
        0x00, 0x00, 0x00, 0x00, 0xaf, 0x8f, // Slot 0x0d, 0x0e and 0x0f, reserved.
    ];

    const TNG_TLS_CHIP_OPTIONS: [u8; Size::Word as usize] = [
        // Index 88..=91, block = 2, offset = 6
        0xff, 0xff, 0x60, 0x0e,
    ];

    const TNG_TLS_KEY_CONFIG_DATA: [u8; Size::Block as usize] = [
        // Index 96..=127, block = 3, offset = 0
        0x53, 0x00, // 0x00
        0x53, 0x00, // 0x01
        0x73, 0x00, 0x73, 0x00, 0x73, 0x00, // 02, 03 and 04
        0x1c, 0x00, // 0x05, reserved.
        0x7c, 0x00, // 0x06
        0x3c, 0x00, // 0x07, reserved.
        0x3c, 0x00, // 0x08
        0x1a, 0x00, // 0x09
        0x1c, 0x00, // 0x0a
        0x10, 0x00, // 0x0b
        0x1c, 0x00, // 0x0c
        0x3c, 0x00, 0x3c, 0x00, 0x1c, 0x00, // 0x0d, 0x0e and 0x0f, reserved.
    ];
}

// Methods for preparing device state. Configuraion, random nonce and key creation and so on.
impl<'a, PHY, D> TrustAndGo<'a, PHY, D>
where
    PHY: i2c::I2c,
    D: DelayNs,
{
    // Slot config
    pub fn configure_permissions(&mut self) -> Result<(), Error> {
        Self::TNG_TLS_SLOT_CONFIG_DATA
            .chunks(Size::Word.len())
            .enumerate()
            .try_for_each(|(i, word)| {
                let index = Memory::<PHY, D>::SLOT_CONFIG_INDEX + i * Size::Word.len();
                let (block, offset, _) = Zone::locate_index(index);
                self.atca
                    .memory()
                    .write_config(Size::Word, block, offset, word)
                    .map(drop)
            })
    }

    // Chip options
    pub fn configure_chip_options(&mut self) -> Result<(), Error> {
        let (block, offset, _) = Zone::locate_index(Memory::<PHY, D>::CHIP_OPTIONS_INDEX);
        self.atca
            .memory()
            .write_config(Size::Word, block, offset, &Self::TNG_TLS_CHIP_OPTIONS)
    }

    // Key config
    pub fn configure_key_types(&mut self) -> Result<(), Error> {
        let (block, offset, _) = Zone::locate_index(Memory::<PHY, D>::KEY_CONFIG_INDEX);
        self.atca
            .memory()
            .write_config(Size::Block, block, offset, &Self::TNG_TLS_KEY_CONFIG_DATA)
    }
}

// On creation of TNG object, enforce stateful configuration.
impl<'a, PHY, D> TryFrom<&'a mut AtCaClient<PHY, D>> for TrustAndGo<'a, PHY, D>
where
    PHY: i2c::I2c,
    D: DelayNs,
{
    type Error = Error;
    fn try_from(atca: &'a mut AtCaClient<PHY, D>) -> Result<Self, Self::Error> {
        let mut tng = Self { atca };
        // Check if configuration zone is locked.
        if !tng.atca.memory().is_locked(Zone::Config)? {
            tng.configure_permissions()?;
            tng.configure_chip_options()?;
            tng.configure_key_types()?;
            // Lock config zone
            tng.atca.memory().lock(Zone::Config)?;
        }

        // Check if data zone is locked.
        if !tng.atca.memory().is_locked(Zone::Data)? {
            // Only lock the data zone for release build
            #[cfg(not(debug_assertions))]
            tng.atca.memory().lock(Zone::Data)?;
        }

        Ok(tng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::OpCode;
    use core::convert::TryInto;
    use core::ops::Deref;
    use heapless::Vec;
    use OpCode::*;
    const KEY_TYPE_P256: u16 = 0x04; // P256 NIST ECC key
    const KEY_TYPE_AES: u16 = 0x06; // AES-128 Key
    const KEY_TYPE_SHA: u16 = 0x07; // SHA key or other data

    struct Provision {
        key_id: Slot,
        permission: u16,
        key_config: u16,
    }

    impl Provision {
        fn new(key_id: Slot) -> Self {
            let permission = permission(key_id);
            let key_config = key_config(key_id);
            Self {
                key_id,
                permission,
                key_config,
            }
        }

        fn is_private(&self) -> bool {
            self.key_config & 0x01 != 0x00
        }

        fn key_type(&self) -> u16 {
            (self.key_config >> 2) & 0x07
        }

        fn read_key(&self) -> u16 {
            self.permission & 0x0f
        }

        fn encrypt_read(&self) -> bool {
            (self.permission >> 6) & 0x01 != 0x00
        }

        fn is_secret(&self) -> bool {
            (self.permission >> 7) & 0x01 != 0x00
        }

        fn write_config(&self) -> u16 {
            (self.permission >> 12) & 0x0f
        }

        fn read_permission(&self) -> &str {
            match (self.is_secret(), self.encrypt_read()) {
                (false, false) => "Clear text",
                (true, false) => "Never",
                (true, true) => "Encrypted",
                _ => panic!("Prohibited"),
            }
        }

        fn write_permission(&self) -> &str {
            match self.write_config() {
                0x00 => "Clear text",
                0x01 => "PubInvalid",
                x if (x >> 1) == 0x01 => "Never",
                x if (x >> 2) == 0x02 => "Never",
                x if (x >> 2) & 0x01 == 0x01 => "Encrypted",
                _ => panic!("Prohibited"),
            }
        }

        // Random nonce
        fn require_nonce(&self) -> bool {
            (self.key_config >> 6) & 0x01 != 0x00
        }

        // Commands that returns its output to the slot.
        fn creation_commands(&self) -> Vec<OpCode, 5> {
            let mut commands = Vec::<OpCode, 5>::new();
            if self.key_id.is_private_key() && self.key_type() == KEY_TYPE_P256 && self.is_secret()
            {
                if (self.write_config() >> 1) & 0x01 == 0x01 {
                    commands.push(GenKey).unwrap();
                    commands.push(DeriveKey).unwrap();
                }
                if (self.write_config() >> 2) & 0x01 == 0x01 {
                    commands.push(PrivWrite).unwrap();
                }
            }
            commands
        }

        // Commands that takes the slot as an intput.
        #[allow(dead_code)]
        fn operation_commands(&self) -> &[OpCode] {
            let mut commands = Vec::<OpCode, 5>::new();
            if self.key_id.is_private_key() {
                if (self.read_key() >> 2) & 0x01 == 0x01 {
                    commands.push(Ecdh).unwrap();
                }
                if self.is_secret() && self.key_type() == KEY_TYPE_P256 {
                    commands.push(Sign).unwrap();
                }
                unimplemented!()
            } else {
                unimplemented!()
            }
        }
    }

    fn permission(key_id: Slot) -> u16 {
        let data = &TrustAndGo::<(), ()>::TNG_TLS_SLOT_CONFIG_DATA;
        let index = key_id as usize * 2;
        let range = index..index + 2;
        data[range]
            .try_into()
            .map(u16::from_le_bytes)
            .unwrap_or_else(|_| unreachable!())
    }

    fn key_config(key_id: Slot) -> u16 {
        let data = &TrustAndGo::<(), ()>::TNG_TLS_KEY_CONFIG_DATA;
        let index = key_id as usize * 2;
        let range = index..index + 2;
        data[range]
            .try_into()
            .map(u16::from_le_bytes)
            .unwrap_or_else(|_| unreachable!())
    }

    // ECC private keys can never be written with the Write and/or DeriveKey
    // commands. Instead, GenKey and PrivWrite can be used to modify these
    // slots.
    #[test]
    fn provision() {
        let auth_priv = Provision::new(AUTH_PRIVATE_KEY);
        assert_eq!(true, auth_priv.is_private());
        assert_eq!(KEY_TYPE_P256, auth_priv.key_type());
        assert_eq!(true, auth_priv.require_nonce());
        assert_eq!(0x05, auth_priv.read_key());
        assert_eq!("Clear text", auth_priv.write_permission());
        assert_eq!(0, auth_priv.creation_commands().len());

        let sign_priv = Provision::new(SIGN_PRIVATE_KEY);
        assert_eq!(true, sign_priv.is_private());
        assert_eq!(KEY_TYPE_P256, sign_priv.key_type());
        assert_eq!(true, sign_priv.require_nonce());
        assert_eq!(0x02, sign_priv.read_key());
        assert_eq!("Clear text", sign_priv.write_permission());
        assert_eq!(0, sign_priv.creation_commands().len());

        for key_id in [USER_PRIVATE_KEY1, USER_PRIVATE_KEY2, USER_PRIVATE_KEY3].iter() {
            let user_priv = Provision::new(*key_id);
            assert_eq!(true, user_priv.is_private());
            assert_eq!(KEY_TYPE_P256, user_priv.key_type());
            assert_eq!(true, user_priv.require_nonce());
            assert_eq!(0x05, user_priv.read_key());
            assert_eq!("Never", user_priv.write_permission());
            assert_eq!(&[GenKey, DeriveKey], user_priv.creation_commands().deref());
        }

        let io_protect = Provision::new(IO_PROTECTION_KEY);
        assert_eq!(KEY_TYPE_SHA, io_protect.key_type());
        assert_eq!("Clear text", io_protect.write_permission());
        assert_eq!(true, io_protect.require_nonce());
        assert_eq!(0, io_protect.creation_commands().len());

        let aes_key = Provision::new(AES_KEY);
        assert_eq!(KEY_TYPE_AES, aes_key.key_type());
        assert_eq!("Never", aes_key.read_permission());
        assert_eq!("Clear text", aes_key.write_permission());
        assert_eq!(0x00, aes_key.write_config());

        let device_cert = Provision::new(DEVICE_CERTIFICATE);
        assert_eq!(KEY_TYPE_SHA, device_cert.key_type());
        assert_eq!("Clear text", device_cert.read_permission());
        assert_eq!("Never", device_cert.write_permission());
        assert_eq!(0x08, device_cert.write_config());

        let signer_pub = Provision::new(SIGNER_PUBLIC_KEY);
        assert_eq!(KEY_TYPE_P256, signer_pub.key_type());
        assert_eq!("Clear text", signer_pub.read_permission());
        assert_eq!("Never", signer_pub.write_permission());
        assert_eq!(0x08, signer_pub.write_config());

        let signer_cert = Provision::new(SIGNER_CERTIFICATE);
        assert_eq!(KEY_TYPE_SHA, signer_cert.key_type());
        assert_eq!("Clear text", signer_cert.read_permission());
        assert_eq!("Never", signer_cert.write_permission());
        assert_eq!(0x08, signer_cert.write_config());
    }
}
