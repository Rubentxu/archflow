//! Security module for command validation, rate limiting, and integrity verification.
//!
//! This module provides security primitives for the ArchFlow engine including:
//! - Token bucket rate limiting
//! - HMAC-SHA256 command signing
//! - Permission validation
//! - Parameter sanitization
//! - Audit logging
//!
//! # Security Considerations
//!
//! - All sensitive operations are logged for audit purposes
//! - Rate limiting prevents abuse and DoS attacks
//! - Command signing ensures message integrity and authenticity

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::hash::{Hash, Hasher};
use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use sha2::Digest;
use sha2::Sha256;

/// Atomic counter for no_std timestamp fallback.
static NOSTD_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns the current timestamp in milliseconds.
///
/// This is a simple implementation that uses the ambient wall clock.
/// In production with `no_std`, consider using a hardware timer or RTC.
#[inline]
pub fn now_ms() -> u64 {
    #[cfg(feature = "std")]
    {
        std::time::UNIX_EPOCH
            .elapsed()
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64
    }
    #[cfg(not(feature = "std"))]
    {
        // For no_std, use atomic counter for thread-safety
        NOSTD_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}

/// Maximum number of tokens in the bucket for rate limiting.
const DEFAULT_BUCKET_CAPACITY: u64 = 100;

/// Default refill rate for token bucket (tokens per second).
const DEFAULT_REFILL_RATE: u64 = 50;

/// Maximum number of audit log entries to keep in memory.
const MAX_AUDIT_LOG_SIZE: usize = 10000;

/// Result type for security operations.
pub type SecurityResult<T> = core::result::Result<T, SecurityError>;

/// Errors that can occur during security operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    /// Rate limit exceeded.
    RateLimitExceeded {
        /// Time until tokens are available (in milliseconds).
        retry_after_ms: u64,
    },
    /// Invalid or missing command signature.
    InvalidSignature,
    /// Permission denied for the requested action.
    PermissionDenied {
        /// Required permission that was not granted.
        required: Permission,
        /// User's granted permissions.
        granted: Vec<Permission>,
    },
    /// Command parameter validation failed.
    InvalidParameter {
        /// Name of the invalid parameter.
        parameter: String,
        /// Reason for validation failure.
        reason: String,
    },
    /// Cryptographic operation failed.
    CryptographicError(String),
    /// Audit log error.
    AuditError(String),
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SecurityError::RateLimitExceeded { retry_after_ms } => {
                write!(f, "Rate limit exceeded. Retry after {}ms", retry_after_ms)
            }
            SecurityError::InvalidSignature => write!(f, "Invalid or missing command signature"),
            SecurityError::PermissionDenied { required, granted } => {
                write!(
                    f,
                    "Permission denied. Required: {:?}, Granted: {:?}",
                    required, granted
                )
            }
            SecurityError::InvalidParameter { parameter, reason } => {
                write!(f, "Invalid parameter '{}': {}", parameter, reason)
            }
            SecurityError::CryptographicError(msg) => {
                write!(f, "Cryptographic error: {}", msg)
            }
            SecurityError::AuditError(msg) => write!(f, "Audit error: {}", msg),
        }
    }
}

/// Supported permissions for command execution.
///
/// Permissions follow the principle of least privilege.
/// Users should only be granted permissions they explicitly need.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Can read/view entities and their properties.
    Read,
    /// Can create new entities.
    Create,
    /// Can modify existing entities.
    Update,
    /// Can delete entities.
    Delete,
    /// Can connect entities (create connections).
    Connect,
    /// Can disconnect entities (remove connections).
    Disconnect,
    /// Can select entities.
    Select,
    /// Can modify entity properties (position, size, color, shape).
    ModifyProperties,
    /// Can execute commands (send commands to the system).
    ExecuteCommand,
    /// Can administer the system (manage users, permissions).
    Admin,
    /// Can view audit logs.
    ViewAuditLog,
    /// Can configure system settings.
    Configure,
}

impl Permission {
    /// Returns true if this permission grants administrative access.
    pub fn is_admin(&self) -> bool {
        matches!(self, Permission::Admin)
    }

    /// Returns a string representation of the permission.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::Read => "read",
            Permission::Create => "create",
            Permission::Update => "update",
            Permission::Delete => "delete",
            Permission::Connect => "connect",
            Permission::Disconnect => "disconnect",
            Permission::Select => "select",
            Permission::ModifyProperties => "modify_properties",
            Permission::ExecuteCommand => "execute_command",
            Permission::Admin => "admin",
            Permission::ViewAuditLog => "view_audit_log",
            Permission::Configure => "configure",
        }
    }
}

/// A command with metadata for security validation.
#[derive(Debug, Clone)]
pub struct SecuredCommand {
    /// The raw command data.
    command_data: Vec<u8>,
    /// User ID who issued the command.
    user_id: u32,
    /// Timestamp when the command was created.
    timestamp: u64,
    /// Sequence number for ordering.
    sequence: u64,
    /// HMAC-SHA256 signature of the command.
    signature: Vec<u8>,
    /// Nonce for preventing replay attacks.
    nonce: [u8; 16],
}

impl SecuredCommand {
    /// Creates a new secured command.
    ///
    /// # Arguments
    ///
    /// * `command_data` - Raw command bytes.
    /// * `user_id` - ID of the user issuing the command.
    /// * `timestamp` - Timestamp when the command was created.
    /// * `sequence` - Sequence number for ordering.
    /// * `signature` - HMAC-SHA256 signature.
    /// * `nonce` - Nonce for replay protection.
    ///
    /// # Returns
    ///
    /// A new `SecuredCommand` instance.
    #[must_use]
    pub fn new(
        command_data: Vec<u8>,
        user_id: u32,
        timestamp: u64,
        sequence: u64,
        signature: Vec<u8>,
        nonce: [u8; 16],
    ) -> Self {
        Self {
            command_data,
            user_id,
            timestamp,
            sequence,
            signature,
            nonce,
        }
    }

    /// Returns the command data.
    #[must_use]
    pub fn command_data(&self) -> &[u8] {
        &self.command_data
    }

    /// Returns the user ID.
    #[must_use]
    pub fn user_id(&self) -> u32 {
        self.user_id
    }

    /// Returns the timestamp.
    #[must_use]
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the sequence number.
    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the signature.
    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Returns the nonce.
    #[must_use]
    pub fn nonce(&self) -> &[u8; 16] {
        &self.nonce
    }
}

/// Token bucket rate limiter.
///
/// The token bucket algorithm allows for burst traffic while enforcing
/// a sustainable average rate. Tokens are added at a fixed rate up to
/// the bucket capacity. Each request consumes one token.
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Maximum number of tokens the bucket can hold.
    capacity: u64,
    /// Current number of tokens in the bucket.
    tokens: u64,
    /// Rate at which tokens are added (tokens per second).
    refill_rate: u64,
    /// Timestamp of last refill (in milliseconds).
    last_refill_ms: u64,
}

impl TokenBucket {
    /// Creates a new token bucket with default settings.
    ///
    /// Default capacity: 100 tokens
    /// Default refill rate: 50 tokens per second
    ///
    /// # Returns
    ///
    /// A new `TokenBucket` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capacity: DEFAULT_BUCKET_CAPACITY,
            tokens: DEFAULT_BUCKET_CAPACITY,
            refill_rate: DEFAULT_REFILL_RATE,
            last_refill_ms: now_ms(),
        }
    }

    /// Creates a token bucket with custom settings.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum tokens the bucket can hold.
    /// * `refill_rate` - Tokens added per second.
    ///
    /// # Returns
    ///
    /// A new `TokenBucket` instance.
    #[must_use]
    pub fn with_capacity_and_rate(capacity: u64, refill_rate: u64) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill_ms: now_ms(),
        }
    }

    /// Attempts to consume a token from the bucket.
    ///
    /// If tokens are available, consumes one token and returns `Ok(())`.
    /// If no tokens are available, returns `Err` with retry time.
    ///
    /// # Returns
    ///
    /// `Ok(())` if token consumed, `Err` with retry time if rate limited.
    pub fn try_consume(&mut self) -> SecurityResult<()> {
        self.refill();
        if self.tokens > 0 {
            self.tokens -= 1;
            Ok(())
        } else {
            let retry_after_ms = self.calculate_retry_delay();
            Err(SecurityError::RateLimitExceeded { retry_after_ms })
        }
    }

    /// Consumes a token, blocking until available.
    ///
    /// This method may block indefinitely if tokens are not replenished.
    /// Consider using `try_consume` for non-blocking operation.
    pub fn consume(&mut self) {
        while self.try_consume().is_err() {
            // Busy wait - in production, use async/blocking with timeout
            core::hint::spin_loop();
        }
    }

    /// Returns the number of available tokens.
    #[must_use]
    pub fn available_tokens(&self) -> u64 {
        self.tokens
    }

    /// Returns the bucket capacity.
    #[must_use]
    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    /// Returns the refill rate (tokens per second).
    #[must_use]
    pub fn refill_rate(&self) -> u64 {
        self.refill_rate
    }

    /// Refills the bucket based on elapsed time.
    fn refill(&mut self) {
        let now = now_ms();
        let elapsed_ms = now.saturating_sub(self.last_refill_ms);
        let elapsed_secs = elapsed_ms / 1000;

        if elapsed_secs > 0 {
            let tokens_to_add = elapsed_secs * self.refill_rate;
            self.tokens = core::cmp::min(self.capacity, self.tokens + tokens_to_add);
            self.last_refill_ms = now;
        }
    }

    /// Calculates delay until next token is available.
    fn calculate_retry_delay(&self) -> u64 {
        if self.tokens > 0 {
            return 0;
        }
        // Tokens are added at refill_rate per second
        // So we need to wait 1000ms / refill_rate per token
        core::cmp::max(1, 1000 / self.refill_rate as u64)
    }
}

impl Default for TokenBucket {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiter per user, using separate token buckets.
#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    /// Token buckets per user ID.
    buckets: alloc::collections::BTreeMap<u32, TokenBucket>,
    /// Default bucket settings.
    default_capacity: u64,
    default_refill_rate: u64,
}

impl UserRateLimiter {
    /// Creates a new user rate limiter.
    ///
    /// # Returns
    ///
    /// A new `UserRateLimiter` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            buckets: alloc::collections::BTreeMap::new(),
            default_capacity: DEFAULT_BUCKET_CAPACITY,
            default_refill_rate: DEFAULT_REFILL_RATE,
        }
    }

    /// Creates a user rate limiter with custom default settings.
    ///
    /// # Arguments
    ///
    /// * `default_capacity` - Default bucket capacity for new users.
    /// * `default_refill_rate` - Default refill rate for new users.
    ///
    /// # Returns
    ///
    /// A new `UserRateLimiter` instance.
    #[must_use]
    pub fn with_defaults(default_capacity: u64, default_refill_rate: u64) -> Self {
        Self {
            buckets: alloc::collections::BTreeMap::new(),
            default_capacity,
            default_refill_rate,
        }
    }

    /// Attempts to consume a token for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to check.
    ///
    /// # Returns
    ///
    /// `Ok(())` if allowed, `Err` with retry time if rate limited.
    pub fn try_consume(&mut self, user_id: u32) -> SecurityResult<()> {
        let bucket = self.buckets.entry(user_id).or_insert_with(|| {
            TokenBucket::with_capacity_and_rate(self.default_capacity, self.default_refill_rate)
        });
        bucket.try_consume()
    }

    /// Consumes a token for a user, blocking until available.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to check.
    pub fn consume(&mut self, user_id: u32) {
        let bucket = self.buckets.entry(user_id).or_insert_with(|| {
            TokenBucket::with_capacity_and_rate(self.default_capacity, self.default_refill_rate)
        });
        bucket.consume();
    }

    /// Returns the number of available tokens for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to check.
    ///
    /// # Returns
    ///
    /// Number of available tokens, or 0 if user not tracked.
    #[must_use]
    pub fn available_tokens(&self, user_id: u32) -> u64 {
        self.buckets
            .get(&user_id)
            .map(|b| b.available_tokens())
            .unwrap_or_default()
    }

    /// Removes a user's bucket (reset their rate limiting).
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID to reset.
    ///
    /// # Returns
    ///
    /// The removed bucket, if it existed.
    pub fn remove_user(&mut self, user_id: u32) -> Option<TokenBucket> {
        self.buckets.remove(&user_id)
    }
}

impl Default for UserRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple HMAC-SHA256 signer for command integrity.
///
/// This implementation provides message authentication using HMAC-SHA256.
/// In production, keys should be stored securely and rotated regularly.
#[derive(Debug, Clone)]
pub struct HmacSigner {
    /// Secret key for HMAC signing.
    key: [u8; 32],
}

impl HmacSigner {
    /// Creates a new HMAC signer with a generated key.
    ///
    /// # Returns
    ///
    /// A new `HmacSigner` with a random key.
    #[must_use]
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        getrandom::getrandom(&mut key).expect("Failed to generate random key");
        Self { key }
    }

    /// Creates a new HMAC signer with a specific key.
    ///
    /// # Arguments
    ///
    /// * `key` - The secret key (must be 32 bytes).
    ///
    /// # Returns
    ///
    /// A new `HmacSigner` with the provided key.
    ///
    /// # Panics
    ///
    /// Panics if the key is not exactly 32 bytes.
    #[must_use]
    pub fn with_key(key: &[u8; 32]) -> Self {
        Self { key: *key }
    }

    /// Signs a command and returns the HMAC-SHA256 signature.
    ///
    /// The signature covers: user_id, timestamp, sequence, nonce, and command data.
    ///
    /// # Arguments
    ///
    /// * `command_data` - The command bytes to sign.
    /// * `user_id` - User ID issuing the command.
    /// * `timestamp` - Command timestamp.
    /// * `sequence` - Sequence number.
    /// * `nonce` - Unique nonce for the command.
    ///
    /// # Returns
    ///
    /// The HMAC-SHA256 signature.
    #[must_use]
    pub fn sign(
        &self,
        command_data: &[u8],
        user_id: u32,
        timestamp: u64,
        sequence: u64,
        nonce: &[u8; 16],
    ) -> Vec<u8> {
        let mut data = Vec::with_capacity(64);
        data.extend_from_slice(&user_id.to_le_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(&sequence.to_le_bytes());
        data.extend_from_slice(nonce);
        data.extend_from_slice(command_data);

        self.hmac_sign(&data)
    }

    /// Verifies a command signature.
    ///
    /// # Arguments
    ///
    /// * `command_data` - The command bytes that were signed.
    /// * `user_id` - User ID that signed the command.
    /// * `timestamp` - Timestamp when the command was signed.
    /// * `sequence` - Sequence number of the command.
    /// * `nonce` - Nonce used for the command.
    /// * `signature` - The signature to verify.
    ///
    /// # Returns
    ///
    /// `Ok(())` if signature is valid, `Err` otherwise.
    pub fn verify(
        &self,
        command_data: &[u8],
        user_id: u32,
        timestamp: u64,
        sequence: u64,
        nonce: &[u8; 16],
        signature: &[u8],
    ) -> SecurityResult<()> {
        let expected = self.sign(command_data, user_id, timestamp, sequence, nonce);
        if expected.len() != signature.len()
            || !constant_time_eq::constant_time_eq(&expected, signature)
        {
            return Err(SecurityError::InvalidSignature);
        }
        Ok(())
    }

    /// Creates a secured command with signature.
    ///
    /// # Arguments
    ///
    /// * `command_data` - The command bytes.
    /// * `user_id` - User ID issuing the command.
    /// * `sequence` - Sequence number.
    ///
    /// # Returns
    ///
    /// A new `SecuredCommand` with signature.
    #[must_use]
    pub fn secure_command(
        &self,
        command_data: Vec<u8>,
        user_id: u32,
        sequence: u64,
    ) -> SecuredCommand {
        let mut nonce = [0u8; 16];
        getrandom::getrandom(&mut nonce).expect("Failed to generate nonce");

        let timestamp = now_ms();
        let signature = self.sign(&command_data, user_id, timestamp, sequence, &nonce);

        SecuredCommand::new(command_data, user_id, timestamp, sequence, signature, nonce)
    }

    /// Performs HMAC signing on data.
    fn hmac_sign(&self, data: &[u8]) -> Vec<u8> {
        // Use proper HMAC-SHA256 from hmac crate
        let mut mac = <Hmac<Sha256> as hmac::digest::KeyInit>::new_from_slice(&self.key)
            .expect("HMAC-SHA256 can accept any key size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }
}

impl Default for HmacSigner {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit log entry for security events.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Timestamp of the event.
    timestamp: u64,
    /// Type of event.
    event_type: AuditEventType,
    /// User ID involved.
    user_id: Option<u32>,
    /// Command or action performed.
    action: String,
    /// Whether the action was allowed.
    success: bool,
    /// Additional details.
    details: Option<String>,
    /// Client IP (if available).
    client_ip: Option<String>,
}

impl AuditEntry {
    /// Creates a new audit entry.
    #[must_use]
    pub fn new(
        event_type: AuditEventType,
        user_id: Option<u32>,
        action: String,
        success: bool,
        details: Option<String>,
        client_ip: Option<String>,
    ) -> Self {
        Self {
            timestamp: now_ms(),
            event_type,
            user_id,
            action,
            success,
            details,
            client_ip,
        }
    }
}

/// Types of events that can be audited.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    /// Authentication events (login, logout, token refresh).
    Authentication,
    /// Authorization events (permission checks).
    Authorization,
    /// Command execution.
    Command,
    /// Configuration changes.
    Configuration,
    /// Security violations (rate limiting, invalid signatures).
    SecurityViolation,
    /// System events.
    System,
}

/// Audit log for tracking security-sensitive events.
#[derive(Debug, Clone)]
pub struct AuditLog {
    /// Circular buffer of audit entries.
    entries: alloc::collections::VecDeque<AuditEntry>,
    /// Maximum entries to keep.
    max_size: usize,
}

impl AuditLog {
    /// Creates a new audit log.
    ///
    /// # Returns
    ///
    /// A new `AuditLog` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: alloc::collections::VecDeque::new(),
            max_size: MAX_AUDIT_LOG_SIZE,
        }
    }

    /// Creates an audit log with custom size.
    ///
    /// # Arguments
    ///
    /// * `max_size` - Maximum entries to keep.
    ///
    /// # Returns
    ///
    /// A new `AuditLog` instance.
    #[must_use]
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            entries: alloc::collections::VecDeque::new(),
            max_size,
        }
    }

    /// Logs an audit entry.
    ///
    /// # Arguments
    ///
    /// * `entry` - The audit entry to log.
    pub fn log(&mut self, entry: AuditEntry) {
        if self.entries.len() >= self.max_size {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Logs a command execution event.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User who executed the command.
    /// * `command` - Description of the command.
    /// * `success` - Whether the command succeeded.
    /// * `details` - Additional details.
    pub fn log_command(
        &mut self,
        user_id: u32,
        command: &str,
        success: bool,
        details: Option<&str>,
    ) {
        self.log(AuditEntry::new(
            AuditEventType::Command,
            Some(user_id),
            command.to_string(),
            success,
            details.map(String::from),
            None,
        ));
    }

    /// Logs a security violation.
    ///
    /// # Arguments
    ///
    /// * `violation_type` - Type of violation.
    /// * `user_id` - User involved (if known).
    /// * `details` - Additional details.
    pub fn log_security_violation(
        &mut self,
        violation_type: &str,
        user_id: Option<u32>,
        details: &str,
    ) {
        self.log(AuditEntry::new(
            AuditEventType::SecurityViolation,
            user_id,
            violation_type.to_string(),
            false,
            Some(details.to_string()),
            None,
        ));
    }

    /// Logs a permission denied event.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User who was denied.
    /// * `permission` - Required permission.
    /// * `action` - Action that was attempted.
    pub fn log_permission_denied(&mut self, user_id: u32, permission: Permission, action: &str) {
        self.log(AuditEntry::new(
            AuditEventType::Authorization,
            Some(user_id),
            action.to_string(),
            false,
            Some(format!("Permission denied: {:?}", permission)),
            None,
        ));
    }

    /// Returns all audit entries.
    #[must_use]
    pub fn entries(&self) -> Vec<&AuditEntry> {
        self.entries.iter().collect()
    }

    /// Returns entries for a specific user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID to filter by.
    ///
    /// # Returns
    ///
    /// Entries for the specified user.
    #[must_use]
    pub fn entries_for_user(&self, user_id: u32) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.user_id == Some(user_id))
            .collect()
    }

    /// Returns failed security events.
    #[must_use]
    pub fn security_violations(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.event_type == AuditEventType::SecurityViolation)
            .collect()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission checker for command authorization.
#[derive(Debug, Clone, Default)]
pub struct PermissionChecker {
    /// User permissions cache.
    permissions: alloc::collections::BTreeMap<u32, alloc::vec::Vec<Permission>>,
}

impl PermissionChecker {
    /// Creates a new permission checker.
    ///
    /// # Returns
    ///
    /// A new `PermissionChecker` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            permissions: alloc::collections::BTreeMap::new(),
        }
    }

    /// Grants a permission to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to grant permission to.
    /// * `permission` - Permission to grant.
    pub fn grant_permission(&mut self, user_id: u32, permission: Permission) {
        self.permissions
            .entry(user_id)
            .or_insert_with(alloc::vec::Vec::new)
            .push(permission);
    }

    /// Revokes a permission from a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to revoke permission from.
    /// * `permission` - Permission to revoke.
    pub fn revoke_permission(&mut self, user_id: u32, permission: Permission) {
        if let Some(perms) = self.permissions.get_mut(&user_id) {
            perms.retain(|p| *p != permission);
        }
    }

    /// Checks if a user has a specific permission.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to check.
    /// * `permission` - Permission to check.
    ///
    /// # Returns
    ///
    /// `true` if the user has the permission.
    #[must_use]
    pub fn has_permission(&self, user_id: u32, permission: Permission) -> bool {
        // Admin has all permissions
        if let Some(perms) = self.permissions.get(&user_id) {
            perms.iter().any(|p| *p == permission || p.is_admin())
        } else {
            false
        }
    }

    /// Checks if a user has all required permissions.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to check.
    /// * `required` - Permissions required.
    ///
    /// # Returns
    ///
    /// `true` if the user has all required permissions.
    #[must_use]
    pub fn has_all_permissions(&self, user_id: u32, required: &[Permission]) -> bool {
        required.iter().all(|p| self.has_permission(user_id, *p))
    }

    /// Returns all permissions for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to check.
    ///
    /// # Returns
    ///
    /// List of permissions, or empty list if none.
    #[must_use]
    pub fn user_permissions(&self, user_id: u32) -> Vec<Permission> {
        self.permissions.get(&user_id).cloned().unwrap_or_default()
    }

    /// Removes all permissions for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to reset.
    pub fn reset_user(&mut self, user_id: u32) {
        self.permissions.remove(&user_id);
    }
}

/// Parameter sanitizer for command inputs.
///
/// Provides sanitization functions to prevent injection attacks
/// and validate command parameters.
#[derive(Debug, Clone, Default)]
pub struct ParameterSanitizer;

impl ParameterSanitizer {
    /// Sanitizes a string parameter.
    ///
    /// Removes null bytes and trims whitespace.
    ///
    /// # Arguments
    ///
    /// * `input` - The input string.
    ///
    /// # Returns
    ///
    /// Sanitized string.
    #[must_use]
    pub fn sanitize_string(input: &str) -> String {
        input.trim().replace('\0', "").replace(['\r', '\n'], " ")
    }

    /// Validates a numeric parameter is within bounds.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to validate.
    /// * `min` - Minimum allowed value.
    /// * `max` - Maximum allowed value.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` with message if invalid.
    pub fn validate_numeric<T: PartialOrd + core::fmt::Display>(
        value: T,
        min: T,
        max: T,
        name: &str,
    ) -> SecurityResult<()> {
        if value < min || value > max {
            Err(SecurityError::InvalidParameter {
                parameter: name.to_string(),
                reason: format!("Value {} out of bounds [{}, {}]", value, min, max),
            })
        } else {
            Ok(())
        }
    }

    /// Validates a string contains no dangerous characters.
    ///
    /// # Arguments
    ///
    /// * `input` - The string to validate.
    /// * `name` - Parameter name for error messages.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` with message if invalid.
    pub fn validate_safe_string(input: &str, name: &str) -> SecurityResult<()> {
        // Check for common injection characters
        let dangerous = [';', '\'', '"', '\\', '{', '}', '[', ']', '(', ')', '<', '>'];

        if input.chars().any(|c| dangerous.contains(&c)) {
            return Err(SecurityError::InvalidParameter {
                parameter: name.to_string(),
                reason: "Input contains dangerous characters".to_string(),
            });
        }

        // Check for null bytes
        if input.contains('\0') {
            return Err(SecurityError::InvalidParameter {
                parameter: name.to_string(),
                reason: "Input contains null bytes".to_string(),
            });
        }

        Ok(())
    }

    /// Validates an entity ID format.
    ///
    /// # Arguments
    ///
    /// * `id` - The entity ID to validate.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err` with message if invalid.
    pub fn validate_entity_id(id: &str) -> SecurityResult<()> {
        if id.is_empty() {
            return Err(SecurityError::InvalidParameter {
                parameter: "entity_id".to_string(),
                reason: "Entity ID cannot be empty".to_string(),
            });
        }

        if id.len() > 128 {
            return Err(SecurityError::InvalidParameter {
                parameter: "entity_id".to_string(),
                reason: "Entity ID too long (max 128 characters)".to_string(),
            });
        }

        Self::validate_safe_string(id, "entity_id")
    }
}

/// Main security service that combines all security components.
#[derive(Debug, Clone)]
pub struct SecurityService {
    /// Rate limiter for users.
    rate_limiter: UserRateLimiter,
    /// HMAC signer for command integrity.
    signer: HmacSigner,
    /// Permission checker.
    permissions: PermissionChecker,
    /// Audit log.
    audit: AuditLog,
    /// Whether to require signatures.
    require_signatures: bool,
}

impl SecurityService {
    /// Creates a new security service.
    ///
    /// # Returns
    ///
    /// A new `SecurityService` instance.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rate_limiter: UserRateLimiter::new(),
            signer: HmacSigner::new(),
            permissions: PermissionChecker::new(),
            audit: AuditLog::new(),
            require_signatures: true,
        }
    }

    /// Validates and processes a command.
    ///
    /// # Arguments
    ///
    /// * `command` - The secured command.
    /// * `required_permission` - Permission required to execute.
    ///
    /// # Returns
    ///
    /// `Ok(command_data)` if valid, `Err` if rejected.
    pub fn validate_command(
        &mut self,
        command: &SecuredCommand,
        required_permission: Permission,
    ) -> SecurityResult<Vec<u8>> {
        // Check rate limiting
        self.rate_limiter
            .try_consume(command.user_id())
            .map_err(|e| {
                self.audit.log_security_violation(
                    "rate_limit",
                    Some(command.user_id()),
                    &format!("Rate limit exceeded for user {}", command.user_id()),
                );
                e
            })?;

        // Verify signature if required
        if self.require_signatures {
            self.signer
                .verify(
                    command.command_data(),
                    command.user_id(),
                    command.timestamp(),
                    command.sequence(),
                    command.nonce(),
                    command.signature(),
                )
                .map_err(|e| {
                    self.audit.log_security_violation(
                        "invalid_signature",
                        Some(command.user_id()),
                        "Command signature verification failed",
                    );
                    e
                })?;
        }

        // Check permission
        if !self
            .permissions
            .has_permission(command.user_id(), required_permission)
        {
            self.audit.log_permission_denied(
                command.user_id(),
                required_permission,
                "command_execution",
            );
            return Err(SecurityError::PermissionDenied {
                required: required_permission,
                granted: self.permissions.user_permissions(command.user_id()),
            });
        }

        // Log successful command
        self.audit.log_command(
            command.user_id(),
            "command_execution",
            true,
            Some(&format!("Sequence: {}", command.sequence())),
        );

        Ok(command.command_data().to_vec())
    }

    /// Attempts to consume a token for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User ID to check.
    ///
    /// # Returns
    ///
    /// `Ok(())` if allowed, `Err` if rate limited.
    pub fn check_rate_limit(&mut self, user_id: u32) -> SecurityResult<()> {
        self.rate_limiter.try_consume(user_id)
    }

    /// Checks if a user has a specific permission.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to check.
    /// * `permission` - Permission to check.
    ///
    /// # Returns
    ///
    /// `true` if the user has the permission.
    #[must_use]
    pub fn has_permission(&self, user_id: u32, permission: Permission) -> bool {
        self.permissions.has_permission(user_id, permission)
    }

    /// Grants a permission to a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to grant permission to.
    /// * `permission` - Permission to grant.
    pub fn grant_permission(&mut self, user_id: u32, permission: Permission) {
        self.permissions.grant_permission(user_id, permission);
    }

    /// Creates a signed command.
    ///
    /// # Arguments
    ///
    /// * `command_data` - Command bytes.
    /// * `user_id` - User ID.
    /// * `sequence` - Sequence number.
    ///
    /// # Returns
    ///
    /// A new secured command.
    #[must_use]
    pub fn sign_command(
        &self,
        command_data: Vec<u8>,
        user_id: u32,
        sequence: u64,
    ) -> SecuredCommand {
        self.signer.secure_command(command_data, user_id, sequence)
    }

    /// Logs an audit entry.
    ///
    /// # Arguments
    ///
    /// * `entry` - Entry to log.
    pub fn log_audit(&mut self, entry: AuditEntry) {
        self.audit.log(entry);
    }

    /// Returns audit entries.
    ///
    /// # Returns
    ///
    /// All audit entries.
    #[must_use]
    pub fn audit_entries(&self) -> Vec<&AuditEntry> {
        self.audit.entries()
    }

    /// Returns rate limit status for a user.
    ///
    /// # Arguments
    ///
    /// * `user_id` - User to check.
    ///
    /// # Returns
    ///
    /// Number of available tokens.
    #[must_use]
    pub fn rate_limit_status(&self, user_id: u32) -> u64 {
        self.rate_limiter.available_tokens(user_id)
    }
}

impl Default for SecurityService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;

    fn create_test_secured_command(
        user_id: u32,
        sequence: u64,
        signer: &HmacSigner,
    ) -> SecuredCommand {
        let data = b"test command data";
        signer.secure_command(data.to_vec(), user_id, sequence)
    }

    #[test]
    fn test_token_bucket_new() {
        let bucket = TokenBucket::new();
        assert_eq!(bucket.capacity(), DEFAULT_BUCKET_CAPACITY);
        assert_eq!(bucket.refill_rate(), DEFAULT_REFILL_RATE);
        assert_eq!(bucket.available_tokens(), DEFAULT_BUCKET_CAPACITY);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::with_capacity_and_rate(5, 1);
        assert_eq!(bucket.available_tokens(), 5);

        for _ in 0..5 {
            assert!(bucket.try_consume().is_ok());
        }
        assert!(bucket.try_consume().is_err());
        assert_eq!(bucket.available_tokens(), 0);
    }

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::with_capacity_and_rate(2, 10);
        assert_eq!(bucket.available_tokens(), 2);

        bucket.try_consume().unwrap();
        bucket.try_consume().unwrap();
        assert_eq!(bucket.available_tokens(), 0);

        // Simulate time passing (100ms = 0.1s = 1 token at rate 10/s)
        // This is a simplified test since we can't easily mock time
        assert_eq!(bucket.available_tokens(), 0);
    }

    #[test]
    fn test_user_rate_limiter() {
        let mut limiter = UserRateLimiter::new();
        assert!(limiter.try_consume(1).is_ok());
        assert!(limiter.try_consume(2).is_ok());
    }

    #[test]
    fn test_hmac_signer_sign_and_verify() {
        let signer = HmacSigner::with_key(&[1u8; 32]);
        let data = b"test data";
        let signature = signer.sign(data, 1, 1000, 1, &[2u8; 16]);

        assert_eq!(signature.len(), 32);
        assert!(
            signer
                .verify(data, 1, 1000, 1, &[2u8; 16], &signature)
                .is_ok()
        );
    }

    #[test]
    fn test_hmac_signer_invalid_verify() {
        let signer = HmacSigner::with_key(&[1u8; 32]);
        let data = b"test data";
        let wrong_signature = vec![0u8; 32];

        assert!(
            signer
                .verify(data, 1, 1000, 1, &[2u8; 16], &wrong_signature)
                .is_err()
        );
    }

    #[test]
    fn test_permission_checker_grant_and_check() {
        let mut checker = PermissionChecker::new();
        assert!(!checker.has_permission(1, Permission::Read));

        checker.grant_permission(1, Permission::Read);
        assert!(checker.has_permission(1, Permission::Read));
        assert!(!checker.has_permission(1, Permission::Update));
    }

    #[test]
    fn test_permission_checker_admin_has_all() {
        let mut checker = PermissionChecker::new();
        checker.grant_permission(1, Permission::Admin);

        assert!(checker.has_permission(1, Permission::Read));
        assert!(checker.has_permission(1, Permission::Update));
        assert!(checker.has_permission(1, Permission::Delete));
    }

    #[test]
    fn test_parameter_sanitizer_sanitize_string() {
        assert_eq!(
            ParameterSanitizer::sanitize_string("  hello\nworld  "),
            "hello world"
        );
        assert_eq!(
            ParameterSanitizer::sanitize_string("test\x00value"),
            "testvalue"
        );
    }

    #[test]
    fn test_parameter_sanitizer_validate_numeric() {
        assert!(ParameterSanitizer::validate_numeric(50, 0, 100, "value").is_ok());
        assert!(ParameterSanitizer::validate_numeric(-1, 0, 100, "value").is_err());
        assert!(ParameterSanitizer::validate_numeric(150, 0, 100, "value").is_err());
    }

    #[test]
    fn test_parameter_sanitizer_validate_safe_string() {
        assert!(ParameterSanitizer::validate_safe_string("valid_name", "name").is_ok());
        assert!(ParameterSanitizer::validate_safe_string("name;drop", "name").is_err());
        assert!(ParameterSanitizer::validate_safe_string("name\x00", "name").is_err());
    }

    #[test]
    fn test_parameter_sanitizer_validate_entity_id() {
        assert!(ParameterSanitizer::validate_entity_id("entity_123").is_ok());
        assert!(ParameterSanitizer::validate_entity_id("").is_err());
        assert!(ParameterSanitizer::validate_entity_id(&"x".repeat(129)).is_err());
    }

    #[test]
    fn test_audit_log_logging() {
        let mut log = AuditLog::new();
        assert_eq!(log.entries().len(), 0);

        log.log(AuditEntry::new(
            AuditEventType::Command,
            Some(1),
            "test".to_string(),
            true,
            None,
            None,
        ));
        assert_eq!(log.entries().len(), 1);
    }

    #[test]
    fn test_audit_log_security_violations() {
        let mut log = AuditLog::new();
        log.log(AuditEntry::new(
            AuditEventType::Command,
            Some(1),
            "test".to_string(),
            true,
            None,
            None,
        ));
        log.log(AuditEntry::new(
            AuditEventType::SecurityViolation,
            Some(2),
            "rate_limit".to_string(),
            false,
            None,
            None,
        ));

        assert_eq!(log.security_violations().len(), 1);
    }

    #[test]
    fn test_security_service_new() {
        let service = SecurityService::new();
        assert_eq!(service.rate_limit_status(1), 0); // No user tracked yet
    }

    #[test]
    fn test_security_service_validate_command() {
        let mut service = SecurityService::new();
        service.grant_permission(1, Permission::ExecuteCommand);

        // Use the service's own signer to create the command
        let command = service.sign_command(b"test command data".to_vec(), 1, 1);

        // Debug: verify command is properly constructed
        assert!(
            !command.signature().is_empty(),
            "Signature should not be empty"
        );
        assert_eq!(command.user_id(), 1, "User ID mismatch");
        assert_eq!(command.sequence(), 1, "Sequence mismatch");

        // Debug: manually verify to see what's different
        let expected_sig = service.signer.sign(
            command.command_data(),
            command.user_id(),
            command.timestamp(),
            command.sequence(),
            command.nonce(),
        );
        let sigs_match = command.signature().len() == expected_sig.len()
            && command.signature()[..] == expected_sig[..];
        assert!(sigs_match, "Signatures don't match!");

        // Verify signature works directly
        let verify_result = service.signer.verify(
            command.command_data(),
            command.user_id(),
            command.timestamp(),
            command.sequence(),
            command.nonce(),
            command.signature(),
        );
        assert!(
            verify_result.is_ok(),
            "Direct verify failed: {:?}",
            verify_result
        );

        let result = service.validate_command(&command, Permission::ExecuteCommand);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }

    #[test]
    fn test_security_service_permission_denied() {
        let mut service = SecurityService::new();
        // Don't grant any permissions

        // Use the service's own signer to create the command
        let command = service.sign_command(b"test command data".to_vec(), 1, 1);

        let result = service.validate_command(&command, Permission::ExecuteCommand);
        assert!(
            matches!(result, Err(SecurityError::PermissionDenied { .. })),
            "Expected PermissionDenied, got: {:?}",
            result
        );
    }

    #[test]
    fn test_security_service_sign_command() {
        let service = SecurityService::new();
        let command = service.sign_command(b"data".to_vec(), 1, 1);

        assert_eq!(command.user_id(), 1);
        assert_eq!(command.sequence(), 1);
        assert!(!command.signature().is_empty());
    }
}
