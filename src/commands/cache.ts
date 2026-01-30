/**
 * Cache Management command types
 */

export interface CacheStats {
  totalEntries: number;
  totalSizeBytes: number;
  entriesByPurpose: Record<string, number>;
  expiredEntries: number;
  oldestEntry?: string;
  newestEntry?: string;
}

export interface CacheCommands {
  get_cache_stats: {
    args: [];
    return: CacheStats;
  };
  clear_cache_by_purpose: {
    args: [purpose: string];
    return: number; // count of deleted entries
  };
  clear_all_cache: {
    args: [];
    return: number; // count of deleted entries
  };
  cleanup_expired_cache: {
    args: [];
    return: number; // count of deleted entries
  };
  evict_cache_by_size: {
    args: [maxSizeMb: number];
    return: number; // count of evicted entries
  };
  evict_cache_by_count: {
    args: [maxEntries: number];
    return: number; // count of evicted entries
  };
}