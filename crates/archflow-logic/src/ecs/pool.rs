// ═══════════════════════════════════════════════════════════════════════════════════════════════════════
// ArchFlow Logic - Memory Pool Module
//
// This module provides memory pooling for ComponentColumn allocations,
// reducing allocation overhead in hot paths during archetype operations.
//
// Key Features:
// - ColumnPool: Reuses Vec<u8> allocations for component storage
// - Zero-copy reuse: Reclaim memory instead of deallocating
// - Type-specific pools: Separate pools per component TypeId
// - Single-threaded: Designed for ECS single-threaded execution model
//
// Performance Benefits:
// - Reduces allocator pressure in hot paths
// - Minimizes heap allocations for recurring component operations
// - Improves cache locality by retaining warm allocations
// - Targets < 1μs for alloc/dealloc operations
//
// Architecture:
// - Pool per TypeId: Components of same type share pool
// - Lazy initialization: Pools created on-demand
// - Capacity tracking: Automatically resizes when exhausted
// - Batch operations: Acquire/release multiple columns efficiently
//
// ═══════════════════════════════════════════════════════════════════════════════════════════════════════


use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::any::TypeId;

use super::component::ComponentId;

/// Memory pool for reusable component column allocations
///
/// Maintains a pool of pre-allocated `Vec<u8>` buffers for each component type.
/// When a column is released, its memory is retained for future allocations,
/// reducing allocator pressure in hot paths.
///
/// # Example
///
/// ```ignore
/// let mut pool = ColumnPool::new();
///
/// // Acquire a column with capacity for 16 elements
/// let column = pool.acquire(ComponentId::of::<Position>(), 16);
///
/// // Use the column...
///
/// // Release back to pool (memory retained)
/// pool.release(ComponentId::of::<Position>(), column);
/// ```
///
/// # Thread Safety
///
/// This pool is designed for single-threaded ECS execution. If thread-safety is needed,
/// wrap the pool in a Mutex or RwLock at the usage site.
#[derive(Debug, Default)]
pub struct ColumnPool {
    /// Per-type storage pools
    pools: BTreeMap<TypeId, TypePool>,
    /// Total allocations made (for monitoring)
    allocations: usize,
    /// Total releases made (for monitoring)
    releases: usize,
}

impl ColumnPool {
    /// Creates a new empty column pool
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            pools: BTreeMap::new(),
            allocations: 0,
            releases: 0,
        }
    }

    /// Acquires a column buffer from the pool or allocates new memory
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component to acquire storage for
    /// - `stride`: Size of each component in bytes
    /// - `capacity`: Minimum number of elements the column should hold
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` with at least `capacity * stride` bytes, either from pool or newly allocated
    #[inline]
    pub fn acquire(
        &mut self,
        component_id: ComponentId,
        stride: usize,
        capacity: usize,
    ) -> Vec<u8> {
        let type_id = component_id.type_id();
        let required_capacity = capacity * stride;

        // Attempt to acquire from pool
        // Get or create pool - ensures pool_count is accurate
        let pool = self.pools.entry(type_id).or_insert_with(TypePool::new);

        if let Some(column) = pool.try_pop(required_capacity) {
            // Reuse from pool - NOT counted as allocation
            return column;
        }

        // Pool exhausted - allocate new
        self.allocations += 1;
        Vec::with_capacity(required_capacity)
    }

    /// Releases a column buffer back to the pool for future reuse
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component the column held
    /// - `column`: The column storage to release (capacity preserved)
    ///
    /// # Notes
    ///
    /// The column's capacity is preserved. If the pool has grown too large,
    /// excess columns are deallocated (up to 16 per type).
    #[inline]
    pub fn release(&mut self, component_id: ComponentId, column: Vec<u8>) {
        let type_id = component_id.type_id();

        // Get or create the type pool
        let pool = self.pools.entry(type_id).or_insert_with(TypePool::new);

        // Try to push, excess will be deallocated
        pool.push(column);

        self.releases += 1;
    }

    /// Returns the total number of allocations made
    #[inline]
    #[must_use]
    pub fn total_allocations(&self) -> usize {
        self.allocations
    }

    /// Returns the total number of releases made
    #[inline]
    #[must_use]
    pub fn total_releases(&self) -> usize {
        self.releases
    }

    /// Tries to acquire a column from the pool
    ///
    /// Unlike [`Self::acquire`], this returns `None` if no suitable column
    /// is available, rather than allocating new memory.
    ///
    /// # Parameters
    ///
    /// - `component_id`: The type of component to acquire storage for
    /// - `stride`: Size of each component in bytes
    /// - `capacity`: Minimum number of elements the column should hold
    ///
    /// # Returns
    ///
    /// `Some(Vec<u8>)` if a suitable column was found, `None` otherwise
    #[inline]
    pub fn try_acquire(
        &mut self,
        component_id: ComponentId,
        stride: usize,
        capacity: usize,
    ) -> Option<Vec<u8>> {
        let type_id = component_id.type_id();
        let required_capacity = capacity * stride;

        if let Some(pool) = self.pools.get_mut(&type_id) {
            if let Some(column) = pool.try_pop(required_capacity) {
                self.allocations += 1;
                return Some(column);
            }
        }

        None
    }

    /// Hit rate (reused allocations / total allocations)
    ///
    /// - `allocations`: Total NEW memory allocations made (not reuse from pool)
    /// - `releases`: Total columns returned to pool
    /// - Hit rate = 1.0 when all columns are reused from pool
    #[inline]
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.allocations == 0 {
            return 1.0;
        }

        // Hit rate = releases / allocations (only new allocations counted)
        self.releases as f64 / self.allocations as f64
    }

    /// Returns the number of active type pools
    #[inline]
    #[must_use]
    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    /// Returns pool statistics
    #[inline]
    #[must_use]
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            allocations: self.allocations,
            releases: self.releases,
            hit_rate_percent: if self.allocations == 0 {
                100.0
            } else {
                (self.releases as f64 / self.allocations as f64 * 100.0) as f32
            },
            pool_count: self.pools.len(),
        }
    }

    /// Clears all pools and deallocates retained memory
    #[inline]
    pub fn clear(&mut self) {
        self.pools.clear();
        self.allocations = 0;
        self.releases = 0;
    }
}

/// Internal pool for a single component type
#[derive(Debug, Default)]
struct TypePool {
    /// Stack of available columns (sorted by capacity for efficient lookup)
    columns: Vec<Vec<u8>>,
    /// Maximum columns to retain per type (prevents unbounded growth)
    max_columns: usize,
}

impl TypePool {
    /// Creates a new empty type pool
    #[inline]
    #[must_use]
    fn new() -> Self {
        Self {
            columns: Vec::new(),
            max_columns: 16,
        }
    }

    /// Tries to pop a column with at least the requested capacity
    ///
    /// Returns `Some(column)` if found, `None` otherwise.
    #[inline]
    fn try_pop(&mut self, min_capacity: usize) -> Option<Vec<u8>> {
        // Search for a column with sufficient capacity
        // Linear search since pools are small (< 16 items)
        for i in 0..self.columns.len() {
            if self.columns[i].capacity() >= min_capacity {
                return Some(self.columns.remove(i));
            }
        }
        None
    }

    /// Pushes a column back into the pool
    ///
    /// If the pool is full, the column is dropped (will be deallocated).
    #[inline]
    fn push(&mut self, column: Vec<u8>) {
        // Only retain if within limits
        if self.columns.len() >= self.max_columns {
            return;
        }

        self.columns.push(column);
    }

    /// Returns the number of columns in the pool
    #[inline]
    #[must_use]
    fn len(&self) -> usize {
        self.columns.len()
    }
}

/// Statistics for monitoring pool performance
#[derive(Debug, Default)]
pub struct PoolStats {
    /// Total allocations made
    pub allocations: usize,
    /// Total releases made
    pub releases: usize,
    /// Hit rate as percentage (0.0 to 100.0)
    pub hit_rate_percent: f32,
    /// Number of active pools
    pub pool_count: usize,
}

impl PoolStats {
    /// Creates stats from a pool
    #[inline]
    #[must_use]
    pub fn from_pool(pool: &ColumnPool) -> Self {
        Self {
            allocations: pool.total_allocations(),
            releases: pool.total_releases(),
            hit_rate_percent: (pool.hit_rate() * 100.0) as f32,
            pool_count: pool.pool_count(),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_pool_acquire_release() {
        let mut pool = ColumnPool::new();

        assert_eq!(pool.total_allocations(), 0);
        assert_eq!(pool.total_releases(), 0);

        // Acquire a column
        let column = pool.acquire(ComponentId::of::<u32>(), 4, 8);

        assert_eq!(column.capacity(), 32); // 8 * 4 bytes
        assert_eq!(column.len(), 0);
        assert_eq!(pool.total_allocations(), 1);

        // Release the column
        pool.release(ComponentId::of::<u32>(), column);

        assert_eq!(pool.total_releases(), 1);
    }

    #[test]
    fn test_column_pool_multiple_types() {
        let mut pool = ColumnPool::new();

        // Acquire columns for different types
        let col1 = pool.acquire(ComponentId::of::<u32>(), 4, 10);
        let col2 = pool.acquire(ComponentId::of::<f32>(), 4, 20);
        let col3 = pool.acquire(ComponentId::of::<u64>(), 8, 5);

        assert_eq!(pool.total_allocations(), 3);

        // Release all
        pool.release(ComponentId::of::<u32>(), col1);
        pool.release(ComponentId::of::<f32>(), col2);
        pool.release(ComponentId::of::<u64>(), col3);

        assert_eq!(pool.total_releases(), 3);
    }

    #[test]
    fn test_pool_hit_rate() {
        let mut pool = ColumnPool::new();

        // Initial state
        assert_eq!(pool.hit_rate(), 1.0);

        // Acquire without release
        let _col = pool.acquire(ComponentId::of::<u32>(), 4, 1);
        assert_eq!(pool.hit_rate(), 0.0);

        // Release and acquire again - should have 100% hit rate
        pool.release(ComponentId::of::<u32>(), _col);
        let _col2 = pool.acquire(ComponentId::of::<u32>(), 4, 1);

        assert!((pool.hit_rate() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_pool_stats() {
        let mut pool = ColumnPool::new();

        // Create some allocations
        let _col1 = pool.acquire(ComponentId::of::<u32>(), 4, 10);
        let _col2 = pool.acquire(ComponentId::of::<f32>(), 4, 20);

        let stats = PoolStats::from_pool(&pool);

        assert_eq!(stats.allocations, 2);
        assert_eq!(stats.pool_count, 2);
    }

    #[test]
    fn test_type_pool_limits() {
        let mut pool = TypePool::new();

        // Fill pool beyond limit (max is 16)
        for _i in 0..20 {
            let col = Vec::with_capacity(100);
            pool.push(col);
        }

        // Only 16 should be retained
        assert_eq!(pool.len(), 16);
    }

    #[test]
    fn test_type_pool_try_pop() {
        let mut pool = TypePool::new();

        // Add columns of different sizes
        pool.push(Vec::with_capacity(50));
        pool.push(Vec::with_capacity(100));
        pool.push(Vec::with_capacity(75));

        // Pop one with minimum capacity 80
        let col = pool.try_pop(80);

        assert!(col.is_some());
        assert!(col.unwrap().capacity() >= 80);
    }

    #[test]
    fn test_column_pool_clear() {
        let mut pool = ColumnPool::new();

        // Create allocations
        let _col1 = pool.acquire(ComponentId::of::<u32>(), 4, 10);
        let _col2 = pool.acquire(ComponentId::of::<f32>(), 4, 20);

        // Clear pool
        pool.clear();

        assert_eq!(pool.total_allocations(), 0);
        assert_eq!(pool.total_releases(), 0);
        assert_eq!(pool.pool_count(), 0);
    }

    #[test]
    fn test_column_pool_reuse() {
        let mut pool = ColumnPool::new();

        // Acquire a column
        let col1 = pool.acquire(ComponentId::of::<u32>(), 4, 16);
        let capacity = col1.capacity();

        // Release it
        pool.release(ComponentId::of::<u32>(), col1);

        // Acquire again - should reuse the same memory
        let col2 = pool.acquire(ComponentId::of::<u32>(), 4, 16);
        assert_eq!(col2.capacity(), capacity);

        // Hit rate should be 100%
        assert!((pool.hit_rate() - 1.0).abs() < 0.001);
    }
}
