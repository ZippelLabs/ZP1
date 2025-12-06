//! Demo program showcasing zp1's accelerated cryptographic precompiles
//! This demonstrates the performance gains from delegation vs pure RISC-V

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Syscall numbers for delegated operations
const KECCAK256_SYSCALL: u32 = 0x1000;
const ECRECOVER_SYSCALL: u32 = 0x1001;
const SHA256_SYSCALL: u32 = 0x1002;
const RIPEMD160_SYSCALL: u32 = 0x1003;

// Simple syscall wrapper
#[inline(always)]
fn syscall(num: u32, arg0: u32, arg1: u32, arg2: u32) -> u32 {
    let result: u32;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a7") num,
            inout("a0") arg0 => result,
            in("a1") arg1,
            in("a2") arg2,
        );
    }
    result
}

// Keccak-256 hash function
fn keccak256(input: &[u8], output: &mut [u8; 32]) {
    syscall(
        KECCAK256_SYSCALL,
        input.as_ptr() as u32,
        input.len() as u32,
        output.as_mut_ptr() as u32,
    );
}

// SHA-256 hash function
fn sha256(input: &[u8], output: &mut [u8; 32]) {
    syscall(
        SHA256_SYSCALL,
        input.as_ptr() as u32,
        input.len() as u32,
        output.as_mut_ptr() as u32,
    );
}

// RIPEMD-160 hash function (returns 20 bytes)
fn ripemd160(input: &[u8], output: &mut [u8; 20]) {
    syscall(
        RIPEMD160_SYSCALL,
        input.as_ptr() as u32,
        input.len() as u32,
        output.as_mut_ptr() as u32,
    );
}

// ECRECOVER signature recovery
fn ecrecover(
    hash: &[u8; 32],
    v: u8,
    r: &[u8; 32],
    s: &[u8; 32],
    output: &mut [u8; 20],
) -> u32 {
    syscall(
        ECRECOVER_SYSCALL,
        hash.as_ptr() as u32,
        ((v as u32) | ((r.as_ptr() as u32) << 8)),
        ((s.as_ptr() as u32) | ((output.as_mut_ptr() as u32) << 16)),
    )
}

#[no_mangle]
pub extern "C" fn _start() {
    // Test data
    let message = b"Hello, zp1 zero-knowledge proof system!";
    
    // Buffer for outputs
    let mut keccak_output = [0u8; 32];
    let mut sha256_output = [0u8; 32];
    let mut ripemd_output = [0u8; 20];
    
    // === Test 1: Keccak-256 ===
    keccak256(message, &mut keccak_output);
    
    // === Test 2: SHA-256 ===
    sha256(message, &mut sha256_output);
    
    // === Test 3: RIPEMD-160 ===
    ripemd160(message, &mut ripemd_output);
    
    // === Test 4: Bitcoin address generation (SHA-256 -> RIPEMD-160) ===
    let pubkey = [0x04u8; 65]; // Dummy uncompressed pubkey
    let mut pubkey_hash = [0u8; 32];
    sha256(&pubkey, &mut pubkey_hash);
    
    let mut bitcoin_address = [0u8; 20];
    ripemd160(&pubkey_hash, &mut bitcoin_address);
    
    // === Test 5: ECRECOVER (Ethereum signature recovery) ===
    let tx_hash: [u8; 32] = [
        0x47, 0x17, 0x32, 0x85, 0xa8, 0xd7, 0x34, 0x1e,
        0x5e, 0x97, 0x2f, 0xc6, 0x77, 0x28, 0x63, 0x29,
        0xc4, 0xb1, 0xf7, 0x43, 0x07, 0x89, 0xa3, 0xb5,
        0xc8, 0x48, 0xc5, 0xf4, 0xd0, 0xef, 0x74, 0x00,
    ];
    
    let v = 28u8;
    
    let r: [u8; 32] = [
        0xb9, 0x1c, 0x75, 0xd3, 0xe8, 0x47, 0xd2, 0x7f,
        0x1f, 0x2a, 0x87, 0x8d, 0x6f, 0x23, 0x58, 0x91,
        0xc5, 0x84, 0x5e, 0x3a, 0x2b, 0x6f, 0xd7, 0x8c,
        0x9a, 0x4f, 0x35, 0x62, 0x1e, 0x8d, 0xa9, 0x3f,
    ];
    
    let s: [u8; 32] = [
        0x3c, 0x8e, 0x65, 0xa7, 0x9f, 0x2c, 0x85, 0x4d,
        0x6a, 0x1b, 0x93, 0xf8, 0x5e, 0x37, 0x4c, 0x62,
        0xd9, 0x18, 0x4f, 0x2e, 0x5c, 0x73, 0xa6, 0x9b,
        0x8d, 0x5f, 0x42, 0x71, 0x3e, 0x9a, 0xb8, 0x4e,
    ];
    
    let mut recovered_address = [0u8; 20];
    let _result = ecrecover(&tx_hash, v, &r, &s, &mut recovered_address);
    
    // Exit successfully
    syscall(93, 0, 0, 0);
}
