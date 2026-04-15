pub mod kunos;
pub mod rfactor2;
pub mod beamng;

use crate::codemasters::CodemastersPacket;

/// Trait that each game adapter implements.
pub trait GameAdapter {
    /// Human-readable name for logging.
    fn name(&self) -> &str;

    /// Try to connect to the game's shared memory.
    /// Returns true if connected, false if SHM not available yet.
    fn connect(&mut self) -> bool;

    /// Check if still connected (SHM still valid).
    fn is_connected(&self) -> bool;

    /// Read current telemetry and pack into a Codemasters UDP packet.
    /// Returns None if no new data since last read.
    fn read(&mut self) -> Option<CodemastersPacket>;

    /// Disconnect and clean up.
    fn disconnect(&mut self);
}
