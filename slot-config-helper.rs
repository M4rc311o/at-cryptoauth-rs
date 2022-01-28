pub mod slot_config {
    use std::convert::TryInto;
    
        pub const SLOT_CONFIG_DATA: [u8; 32] = [
            // Index 20..=51, block = 0, offset = 5
            0x85, 0x00, // Slot 0x00, Primary private key
            0x82, 0x00, // Slot 0x01, Internal sign private key
            0x85, 0x20, // Slot 0x02, Secondary private key 1
            0x85, 0x20, // Slot 0x03, Secondary private key 2
            0x85, 0x20, // Slot 0x04, Secondary private key 3
            0x8f, 0x8f, // Slot 0x05, reserved
            0x8f, 0x0f, // Slot 0x06, I/O protection key
            0xaf, 0x8f, // Slot 0x07, reserved
            0x0f, 0x0f, // Slot 0x08, General data
            0x8f, 0x0f, // Slot 0x09, AES key
            0x0f, 0x8f, // Slot 0x0a, Device compressed certificate
            0x0f, 0x8f, // Slot 0x0b, Signer public key
            0x0f, 0x8f, // Slot 0x0c, Signer compressed certificate
            0x00, 0x00, // Slot 0x0d,
            0x00, 0x00, // Slot 0x0e,
            0xaf, 0x8f, // Slot 0x0f,
        ];
        
        /// Use this keyID to encrypt data being read from this slot using the Read command. See more
        /// information in the description for bit 6 in Table 2-6.
        /// 0 = Then this slot can be the source for the CheckMac copy operation. See Section 4.4.6,
        /// Password Checking.
        /// ► Do not use zero as a default. Do not set this field to zero unless the CheckMac copy
        /// operation is explicitly desired, regardless of any other read/write restrictions.
        /// Slots containing private keys can never be read and this field has a different meaning:
        /// Bit 0: External signatures of arbitrary messages are enabled.
        /// Bit 1: Internal signatures of messages generated by GenDig or GenKey are enabled.
        /// Bit 2: ECDH operation is permitted for this key.
        /// Bit 3: If clear, then ECDH master secret will be output in the clear. If set, then master secret will
        /// be written into slot N|1. Ignored if Bit 2 is zero.
        /// For slots containing public keys that can be validated (PubInfo is one, see Section 2.2.11, KeyConfig),
        /// this field stored the ID of the key that should be used to perform the validation.
        fn read_key(slot_config: u16) {
            println!("  Read Key: {:#06b}", slot_config & 0xF);
            if (slot_config >> 0) & 1 == 1 {
                println!("    - External signatures of arbitrary messages are enabled");
            }
            if (slot_config >> 1) & 1 == 1 {
                println!("    - Internal signatures of messages generated by GenDig or GenKey are enabled");
            }
            if (slot_config >> 2) & 1 == 1 {
                println!("    - ECDH operation is permitted for this key");
            } else {
                if (slot_config >> 3) & 1 == 1 {
                    println!("    - master secret will be written into slot N|1");
                }
            }
        }
        
        /// 1 = The key stored in the slot is intended for verification usage and cannot be used by the MAC
        /// command. When this key is used to generate or modify TempKey, then that value may not be
        /// used by the MAC command.
        /// 0 = The key stored in the slot can be used by all commands.
        fn no_mac(slot_config: u16) {
            println!("  No MAC: {:#b}", slot_config & 0x1);
            if (slot_config >> 0) & 1 == 1 {
                println!("    - The key stored in the slot is intended for verification usage and cannot be used by the MAC command");
            } else {
                println!("    - The key stored in the slot can be used by all commands");
            }
        }
        
        /// 1 = The key stored in the slot is “Limited Use” and its use is controlled by Counter0. See Section 4.4.5,
        /// High Endurance Monotonic Counters.
        /// 0 = There are no usage limitations.
        fn limited_use(slot_config: u16) {
            println!("  Limited Use: {:#b}", slot_config & 0x1);
            if (slot_config >> 0) & 1 == 1 {
                println!(
                    "    - The key stored in the slot is “Limited Use” and its use is controlled by Counter0"
                );
            } else {
                println!("    - There are no usage limitations");
            }
        }
        
        /// 1 = Reads from this slot will be encrypted using the procedure specified in the Read
        /// command using ReadKey (bits 0 – 3 in this table) to generate the encryption key. No input
        /// MAC is required. If this bit is set, then IsSecret must also be set (in addition, see the following
        /// Table 2-6).
        /// 0 = Clear text reads may be permitted.
        fn encrypt_read(slot_config: u16) {
            println!("  Encrypt Read: {:#b}", slot_config & 0x1);
            if (slot_config >> 0) & 1 == 1 {
                println!("    - Reads from this slot will be encrypted using the procedure specified in the Read command using ReadKey");
            } else {
                println!("    - Clear text reads may be permitted");
            }
        }
        
        /// 1 = The contents of this slot are secret – Clear text reads are prohibited and both 4-byte reads and
        /// writes are prohibited. This bit must be set if EncryptRead is a one or if WriteConfig has any value
        /// other than Always to ensure proper operation of the device.
        /// 0 = The contents of this slot should contain neither confidential data nor keys. The GenKey and
        /// Sign commands will fail if IsSecret is set to zero for any ECC private key.
        /// See Table 2-6 for additional information.
        fn is_secret(slot_config: u16) {
            println!("  Is Secret: {:#b}", slot_config & 0x1);
            if (slot_config >> 0) & 1 == 1 {
                println!("    - The contents of this slot are secret");
            } else {
                println!("    - The contents of this slot should contain neither confidential data nor keys");
            }
        }
        
        /// Use this key to validate and encrypt data written to this slot
        fn write_key(slot_config: u16) {
            println!("  Write Key: {:#04x}", slot_config & (0xF));
        }
        
        /// Controls the ability to modify the data in this slot.
        /// See Table 2-7, Table 2-8, Table 2-10, and 11.23.
        fn write_config(slot_config: u16) {
            println!("  Write Config: {:#04x}", slot_config & (0xF));
        }
    
        pub fn print() {
            SLOT_CONFIG_DATA.chunks(2).enumerate().for_each(|(i, word)| {
                let slot_config = u16::from_le_bytes(word.try_into().unwrap());
                println!("Slot {} [{:#018b}]:", i, slot_config);
        
                read_key(slot_config >> 0);
                no_mac(slot_config >> 4);
                limited_use(slot_config >> 5);
                encrypt_read(slot_config >> 6);
                is_secret(slot_config >> 7);
                write_key(slot_config >> 8);
                write_config(slot_config >> 12);
                
                println!("\r\n\r\n");
            })
        }
    }
    
    pub mod key_config {
    use std::convert::TryInto;
        pub const KEY_CONFIG_DATA: [u8; 32] = [
            // Index 96..=127, block = 3, offset = 0
            0x53, 0x00, // 0x00, Primary private key ---- til provision private key
            0x53, 0x00, // 0x01, Internal sign private key ---- bare reserved hvis vi vil have en separat sign pk?
            0x73, 0x00, // 0x02, Secondary private key 1   ---- hvad vil vi gemme her?
            0x73, 0x00, // 0x03, Secondary private key 2   ---- hvad vil vi gemme her?
            0x73, 0x00, // 0x04, Secondary private key 3   ---- hvad vil vi gemme her?
            0x1c, 0x00, // 0x05, General data, not lockable---- hvad vil vi gemme her?
            0x7c, 0x00, // 0x06, I/O protection key        ---- til at udvide til encrypted io i fremtiden?
            0x3c, 0x00, // 0x07, General data, lockable    ---- hvad vil vi gemme her?
            0x3c, 0x00, // 0x08, General data, lockable    ---- hvad vil vi gemme her?
            0x1a, 0x00, // 0x09, AES key                   ---- kan gemme 
            0x1c, 0x00, // 0x0a, Device compressed certificate ---- altså det er aws certificate?
            0x10, 0x00, // 0x0b, Signer public key             ---- hvad vil vi gemme her?
            0x1c, 0x00, // 0x0c, Signer compressed certificate ---- hvad vil vi gemme her?
            0x3c, 0x00, // 0x0d, General data, lockable        ---- hvad vil vi gemme her?
            0x3c, 0x00, // 0x0e, General data, lockable        ---- hvad vil vi gemme her?
            0x1c, 0x00, // 0x0f, General data, not lockable    ---- hvad vil vi gemme her?
        ];
    
        fn private(slot_config: u16) {
            println!("  Private: ");
            if (slot_config >> 1) & 1 == 1 {
                println!("    - The key slot contains an ECC private key and can be accessed only with the Sign, GenKey, and PrivWrite commands");
                pub_info(slot_config >> 1, 1);
            } else {
                println!("    - The key slot does not contain an ECC private key and cannot be accessed with the Sign, GenKey, and PrivWrite commands. It may contain an ECC public key, a SHA key, or data");
                pub_info(slot_config >> 1, 0);

            }
        }
        
        fn pub_info(slot_config: u16, private: u8) {
            println!("  Pub Info: ");
            match (private, (slot_config >> 1) & 1) {
                (1, 1) => println!("    - The public version of this key can always be generated."),
                (1, 0) => println!("    - The public version of this key can never be generated. Use this mode for the highest security."),
                (0, 1) => println!("    - = The public key in this slot can be used by the Verify command only if the public key in the slot has been validated."),
                (0, 0) => println!("    - The public key in this slot can be used by the Verify command without being validated."),
                _ => {}
            }
        }
    
        fn key_type(slot_config: u16) { 
            match slot_config & 0xF {
                4 => println!("  Key Type = P256 NIST ECC key"),
                6 => println!("  Key Type = AES key"),
                7 => println!("  Key Type = SHA key or other data"),
                _ => println!("  Key Type = RFU (reserved for future use)"),
            }
        }
    
        fn lockable(slot_config: u16) { 
            println!("  Lockable: ");
            if (slot_config >> 1) & 1 == 1 {
                println!("    - This slot can be individually locked using the Lock command");
            } else {
                println!("    - The remaining keyConfig and slotConfig bits control modification permission");
            }
        }
    
        fn req_random(slot_config: u16) { 
            println!("  Req Random: ");
            if (slot_config >> 1) & 1 == 1 {
                println!("    - A random nonce is required");
            } else {
                println!("    - A random nonce is not required");
            }
        }
    
        fn req_auth(slot_config: u16) { 
            println!("  Req Auth: ");
            if (slot_config >> 1) & 1 == 1 {
                println!("    - Before this key must be used, a prior authorization using the key pointed to by AuthKey must be completed successfully prior to cryptographic use of the key");
                auth_key(slot_config);
            } else {
                println!("    - No prior authorization is required");
            }
        }
        
        fn auth_key(slot_config: u16) { 
            println!("      Auth Key: {:#04x}", slot_config & 0xF);
        }
    
        fn persistent_disable(slot_config: u16) { 
            println!("  Persistent Disable: ");
            if (slot_config >> 1) & 1 == 1 {
                println!("    - Use of this key is prohibited for all commands other than GenKey if the PersistentLatch is zero");
            } else {
                println!("    - Then use of this key is independent of the state of the PersistentLatch.");
            }
        }
    
        fn x509id(slot_config: u16) { 
            println!("  X509 ID: {:#04x}", slot_config & 0b11);
        }
    
        pub fn print() {
            KEY_CONFIG_DATA.chunks(2).enumerate().for_each(|(i, word)| {
                let key_config = u16::from_le_bytes(word.try_into().unwrap());
                println!("Slot {} [{:#018b}]:", i, key_config);
        
                private(key_config >> 0);
                key_type(key_config >> 2);
                lockable(key_config >> 5);
                req_random(key_config >> 6);
                req_auth(key_config >> 7);
                persistent_disable(key_config >> 12);
                x509id(key_config >> 14);
                
                println!("\r\n\r\n");
            })
        }
    }
    
    
    
    
    fn main() {
        slot_config::print();
        key_config::print();
    }
    