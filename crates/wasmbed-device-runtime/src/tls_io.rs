// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! I/O layer for TLS in no_std environments
//! Provides Read/Write traits compatible with embedded-tls

extern crate alloc;

use core::fmt;

#[cfg(feature = "tls")]
use embedded_io::{ErrorKind, ErrorType, Read as EmbeddedRead, Write as EmbeddedWrite};

#[cfg(not(feature = "tls"))]
// Simplified traits when TLS is not enabled
pub trait Read {
    type Error;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

#[cfg(not(feature = "tls"))]
pub trait Write {
    type Error;
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;
    fn flush(&mut self) -> Result<(), Self::Error>;
}

#[cfg(feature = "tls")]
pub use embedded_io::{Read, Write};

/// Error type for I/O operations compatible with embedded-io
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IoError {
    WouldBlock,
    UnexpectedEof,
    InvalidInput,
    Other,
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::WouldBlock => write!(f, "WouldBlock"),
            IoError::UnexpectedEof => write!(f, "UnexpectedEof"),
            IoError::InvalidInput => write!(f, "InvalidInput"),
            IoError::Other => write!(f, "Other"),
        }
    }
}

#[cfg(feature = "tls")]
impl ErrorType for IoError {
    type Error = IoError;
}

#[cfg(feature = "tls")]
impl embedded_io::Error for IoError {
    fn kind(&self) -> embedded_io::ErrorKind {
        // embedded_io::ErrorKind only has Other variant in version 0.6
        embedded_io::ErrorKind::Other
    }
}

/// In-memory I/O buffer for testing/simulation
/// In production, this would be replaced with actual network I/O
pub struct MemoryIo {
    read_buffer: heapless::Vec<u8, 4096>,
    write_buffer: heapless::Vec<u8, 4096>,
    read_pos: usize,
}

impl MemoryIo {
    pub fn new() -> Self {
        Self {
            read_buffer: heapless::Vec::new(),
            write_buffer: heapless::Vec::new(),
            read_pos: 0,
        }
    }

    pub fn push_read_data(&mut self, data: &[u8]) -> Result<(), IoError> {
        for &byte in data {
            if self.read_buffer.push(byte).is_err() {
                return Err(IoError::Other);
            }
        }
        Ok(())
    }

    pub fn take_write_data(&mut self) -> heapless::Vec<u8, 4096> {
        let data = self.write_buffer.clone();
        self.write_buffer.clear();
        data
    }

    pub fn has_data(&self) -> bool {
        self.read_pos < self.read_buffer.len()
    }
}

/// Network I/O layer for real TLS connections
/// 
/// Supports both std (with std::net::TcpStream) and no_std (with smoltcp or custom stack)
#[cfg(feature = "std")]
pub struct NetworkIo {
    #[cfg(feature = "std")]
    stream: Option<std::net::TcpStream>,
    endpoint: heapless::String<64>,
    connected: bool,
}

#[cfg(not(feature = "std"))]
pub struct NetworkIo {
    // For no_std, use shared memory for TCP bridge communication
    // Memory addresses for TCP bridge communication (shared with host)
    // 0x20002000: TCP connection status (0 = disconnected, 1 = connected)
    // 0x20002004: TCP read buffer address
    // 0x20002008: TCP read buffer length
    // 0x2000200C: TCP write buffer address
    // 0x20002010: TCP write buffer length
    // 0x20002014-0x20002114: TCP read buffer (256 bytes)
    // 0x20002114-0x20002214: TCP write buffer (256 bytes)
    endpoint: heapless::String<64>,
    connected: bool,
    read_pos: usize,
    write_pos: usize,
}

impl NetworkIo {
    pub fn new(endpoint: &str) -> Result<Self, IoError> {
        let mut ep = heapless::String::new();
        ep.push_str(endpoint).map_err(|_| IoError::InvalidInput)?;
        
        #[cfg(feature = "std")]
        {
            Ok(Self {
                stream: None,
                endpoint: ep,
                connected: false,
            })
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, use shared memory for TCP bridge communication
            Ok(Self {
                endpoint: ep,
                connected: false,
                read_pos: 0,
                write_pos: 0,
            })
        }
    }

    pub fn connect(&mut self) -> Result<(), IoError> {
        #[cfg(feature = "std")]
        {
            // Parse endpoint (host:port)
            let endpoint_str = self.endpoint.as_str();
            let addr: std::net::SocketAddr = endpoint_str
                .parse()
                .map_err(|_| IoError::InvalidInput)?;
            
            // Create TCP socket and connect
            let stream = std::net::TcpStream::connect(addr)
                .map_err(|_| IoError::Other)?;
            
                    // Set blocking mode for tests (non-blocking causes WouldBlock errors)
                    // In production, this could be non-blocking with proper error handling
                    stream.set_nonblocking(false)
                        .map_err(|_| IoError::Other)?;
            
            self.stream = Some(stream);
            self.connected = true;
            Ok(())
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, use syscall to connect to TCP bridge
            // Write endpoint to shared memory and trigger connection
            // The host (Renode) will read the endpoint and establish TCP connection to bridge
            
            use log::{info, warn, error};
            
            info!("NetworkIo::connect() - Writing endpoint to shared memory: {}", self.endpoint);
            
            // Write endpoint to memory at 0x20002000
            // Format: [status: u32][endpoint_len: u32][endpoint_bytes...]
            unsafe {
                let status_ptr = 0x20002000 as *mut u32;
                let endpoint_len_ptr = 0x20002004 as *mut u32;
                let endpoint_bytes_ptr = 0x20002008 as *mut u8;
                
                // Write endpoint length and bytes
                *endpoint_len_ptr = self.endpoint.len() as u32;
                let endpoint_bytes = self.endpoint.as_bytes();
                info!("NetworkIo::connect() - Writing {} bytes to memory", endpoint_bytes.len());
                for (i, &byte) in endpoint_bytes.iter().enumerate() {
                    if i < 64 {
                        *endpoint_bytes_ptr.add(i) = byte;
                    }
                }
                
                // Write connection request (1 = connect)
                *status_ptr = 1;
                info!("NetworkIo::connect() - Connection request written (status=1), waiting for host...");
            }
            
            // Wait for connection to be established (host sets status to 2 = connected)
            // Simple polling loop (in production, use interrupts or better mechanism)
            let max_attempts = 100000;
            for attempt in 0..max_attempts {
                unsafe {
                    let status_ptr = 0x20002000 as *const u32;
                    let status = *status_ptr;
                    if status == 2 {
                        // Connected
                        info!("NetworkIo::connect() - ✅ Connection established! (status=2, attempt={})", attempt);
                        self.connected = true;
                        self.read_pos = 0;
                        self.write_pos = 0;
                        return Ok(());
                    } else if attempt % 10000 == 0 && attempt > 0 {
                        warn!("NetworkIo::connect() - Still waiting for connection (status={}, attempt={}/{})", status, attempt, max_attempts);
                    }
                }
                // Simple delay
                for _ in 0..1000 {
                    core::hint::spin_loop();
                }
            }
            
            // Connection timeout
            error!("NetworkIo::connect() - ❌ Connection timeout after {} attempts (status never became 2)", max_attempts);
            error!("NetworkIo::connect() - NOTE: TCP bridge may not be running. Check Renode TCP bridge configuration.");
            Err(IoError::Other)
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(feature = "tls")]
impl ErrorType for NetworkIo {
    type Error = IoError;
}

#[cfg(feature = "tls")]
impl embedded_io::Read for NetworkIo {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.connected {
            return Err(IoError::Other);
        }
        
        #[cfg(feature = "std")]
        {
            if let Some(ref mut stream) = self.stream {
                use std::io::Read as StdRead;
                match stream.read(buf) {
                    Ok(0) => Err(IoError::UnexpectedEof),
                    Ok(n) => Ok(n),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(IoError::WouldBlock),
                    Err(_) => Err(IoError::Other),
                }
            } else {
                Err(IoError::Other)
            }
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, read from shared memory TCP buffer
            // Host (Renode) writes data to 0x20002014-0x20002114
            unsafe {
                let read_buffer_ptr = 0x20002014 as *const u8;
                let read_length_ptr = 0x20002008 as *mut u32;
                
                // Read available data length
                let available = *read_length_ptr as usize;
                
                if self.read_pos >= available {
                    // No more data available
                    return Err(IoError::WouldBlock);
                }
                
                // Read data from buffer
                let to_read = buf.len().min(available - self.read_pos);
                for i in 0..to_read {
                    buf[i] = *read_buffer_ptr.add(self.read_pos + i);
                }
                
                self.read_pos += to_read;
                
                // If we've read all available data, reset position and clear length
                if self.read_pos >= available {
                    self.read_pos = 0;
                    *read_length_ptr = 0;
                }
                
                Ok(to_read)
            }
        }
    }
}

#[cfg(feature = "tls")]
impl embedded_io::Write for NetworkIo {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if !self.connected {
            return Err(IoError::Other);
        }
        
        #[cfg(feature = "std")]
        {
            if let Some(ref mut stream) = self.stream {
                use std::io::Write as StdWrite;
                match stream.write(buf) {
                    Ok(n) => Ok(n),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(IoError::WouldBlock),
                    Err(_) => Err(IoError::Other),
                }
            } else {
                Err(IoError::Other)
            }
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, write to shared memory TCP buffer
            // Host (Renode) reads data from 0x20002114-0x20002214 and forwards to bridge
            unsafe {
                let write_buffer_ptr = 0x20002114 as *mut u8;
                let write_length_ptr = 0x2000200C as *mut u32;
                let write_status_ptr = 0x20002010 as *mut u32;
                
                // Check if buffer has space (max 256 bytes)
                let current_length = *write_length_ptr as usize;
                let available_space = 256 - current_length;
                
                if available_space == 0 {
                    // Buffer full, wait for host to read
                    return Err(IoError::WouldBlock);
                }
                
                // Write data to buffer
                let to_write = buf.len().min(available_space);
                for i in 0..to_write {
                    *write_buffer_ptr.add(current_length + i) = buf[i];
                }
                
                // Update length and set write flag (1 = data ready)
                *write_length_ptr = (current_length + to_write) as u32;
                *write_status_ptr = 1;
                
                Ok(to_write)
            }
        }
    }

    fn flush(&mut self) -> Result<(), <Self as embedded_io::ErrorType>::Error> {
        #[cfg(feature = "std")]
        {
            if let Some(ref mut stream) = self.stream {
                use std::io::Write as StdWrite;
                stream.flush().map_err(|_| IoError::Other)
            } else {
                Ok(())
            }
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, flush is handled by host (Renode) reading from shared memory
            // No-op here, host will read when write_status is set
            Ok(())
        }
    }
}

#[cfg(not(feature = "tls"))]
impl Read for MemoryIo {
    type Error = IoError;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.read_pos >= self.read_buffer.len() {
            return Err(IoError::WouldBlock);
        }

        let available = self.read_buffer.len() - self.read_pos;
        let to_read = buf.len().min(available);
        
        buf[..to_read].copy_from_slice(
            &self.read_buffer[self.read_pos..self.read_pos + to_read]
        );
        self.read_pos += to_read;
        
        Ok(to_read)
    }
}

#[cfg(not(feature = "tls"))]
impl Write for MemoryIo {
    type Error = IoError;
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for &byte in buf {
            if self.write_buffer.push(byte).is_err() {
                return Err(IoError::Other);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // No-op for in-memory buffer
        Ok(())
    }
}

#[cfg(feature = "tls")]
impl ErrorType for MemoryIo {
    type Error = IoError;
}

#[cfg(feature = "tls")]
impl embedded_io::Read for MemoryIo {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if self.read_pos >= self.read_buffer.len() {
            return Err(IoError::WouldBlock);
        }

        let available = self.read_buffer.len() - self.read_pos;
        let to_read = buf.len().min(available);
        
        buf[..to_read].copy_from_slice(
            &self.read_buffer[self.read_pos..self.read_pos + to_read]
        );
        self.read_pos += to_read;
        
        Ok(to_read)
    }
}

#[cfg(feature = "tls")]
impl embedded_io::Write for MemoryIo {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        for &byte in buf {
            if self.write_buffer.push(byte).is_err() {
                return Err(IoError::Other);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        // No-op for in-memory buffer
        Ok(())
    }
}

#[cfg(not(feature = "tls"))]
impl Read for NetworkIo {
    type Error = IoError;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if !self.connected {
            return Err(IoError::Other);
        }
        
        #[cfg(feature = "std")]
        {
            if let Some(ref mut stream) = self.stream {
                use std::io::Read as StdRead;
                match stream.read(buf) {
                    Ok(0) => Err(IoError::UnexpectedEof),
                    Ok(n) => Ok(n),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(IoError::WouldBlock),
                    Err(_) => Err(IoError::Other),
                }
            } else {
                Err(IoError::Other)
            }
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, read from shared memory TCP buffer
            unsafe {
                let read_buffer_ptr = 0x20002014 as *const u8;
                let read_length_ptr = 0x20002008 as *mut u32;
                
                let available = *read_length_ptr as usize;
                
                if self.read_pos >= available {
                    return Err(IoError::WouldBlock);
                }
                
                let to_read = buf.len().min(available - self.read_pos);
                for i in 0..to_read {
                    buf[i] = *read_buffer_ptr.add(self.read_pos + i);
                }
                
                self.read_pos += to_read;
                
                if self.read_pos >= available {
                    self.read_pos = 0;
                    *read_length_ptr = 0;
                }
                
                Ok(to_read)
            }
        }
    }
}

#[cfg(not(feature = "tls"))]
impl Write for NetworkIo {
    type Error = IoError;
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        if !self.connected {
            return Err(IoError::Other);
        }
        
        #[cfg(feature = "std")]
        {
            if let Some(ref mut stream) = self.stream {
                use std::io::Write as StdWrite;
                match stream.write(buf) {
                    Ok(n) => Ok(n),
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Err(IoError::WouldBlock),
                    Err(_) => Err(IoError::Other),
                }
            } else {
                Err(IoError::Other)
            }
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, write to shared memory TCP buffer
            unsafe {
                let write_buffer_ptr = 0x20002114 as *mut u8;
                let write_length_ptr = 0x2000200C as *mut u32;
                let write_status_ptr = 0x20002010 as *mut u32;
                
                let current_length = *write_length_ptr as usize;
                let available_space = 256 - current_length;
                
                if available_space == 0 {
                    return Err(IoError::WouldBlock);
                }
                
                let to_write = buf.len().min(available_space);
                for i in 0..to_write {
                    *write_buffer_ptr.add(current_length + i) = buf[i];
                }
                
                *write_length_ptr = (current_length + to_write) as u32;
                *write_status_ptr = 1;
                
                Ok(to_write)
            }
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        #[cfg(feature = "std")]
        {
            if let Some(ref mut stream) = self.stream {
                use std::io::Write as StdWrite;
                stream.flush().map_err(|_| IoError::Other)
            } else {
                Ok(())
            }
        }
        
        #[cfg(not(feature = "std"))]
        {
            // For no_std, flush is handled by host (Renode) reading from shared memory
            Ok(())
        }
    }
}

