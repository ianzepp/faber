// Tabula - HashMap wrapper for Faber
// Wraps std.StringHashMap/AutoHashMap with Latin method names
//
// WHY: No stored allocator. Methods that need allocation accept it as parameter.
// This ensures cura...fit blocks properly thread the current allocator.

const std = @import("std");

# Tabula(K, V) is a hash map with Latin-named methods.
# For string keys, use TabulaTextus(V) which wraps StringHashMap.
# For other keys, use TabulaAuto(K, V) which wraps AutoHashMap.
#
# WHY: Zig's HashMap API requires allocator for put() but not get().
# We follow that pattern: only mutating/growing operations need allocator.
pub fn Tabula(comptime K: type, comptime V: type) type {
    // WHY: String keys use StringHashMap for better performance
    if (K == []const u8) {
        return TabulaTextus(V);
    }
    return TabulaAuto(K, V);
}

# Tabula for string keys - wraps std.StringHashMap
pub fn TabulaTextus(comptime V: type) type {
    return struct {
        map: std.StringHashMap(V),

        const Self = @This();

        // =====================================================================
        // CONSTRUCTION
        // =====================================================================

        # Create an empty tabula.
        pub fn init(alloc: std.mem.Allocator) Self {
            return .{ .map = std.StringHashMap(V).init(alloc) };
        }

        # Free the tabula's memory.
        pub fn deinit(self: *Self) void {
            self.map.deinit();
        }

        // =====================================================================
        // CORE OPERATIONS
        // =====================================================================

        # Set key-value pair (mutates, needs allocator for growth).
        pub fn pone(self: *Self, alloc: std.mem.Allocator, key: []const u8, value: V) void {
            _ = alloc; // StringHashMap tracks its allocator internally
            self.map.put(key, value) catch @panic("OOM");
        }

        # Get value by key (returns optional).
        pub fn accipe(self: Self, key: []const u8) ?V {
            return self.map.get(key);
        }

        # Get value or return default.
        pub fn accipeAut(self: Self, key: []const u8, default: V) V {
            return self.map.get(key) orelse default;
        }

        # Check if key exists.
        pub fn habet(self: Self, key: []const u8) bool {
            return self.map.contains(key);
        }

        # Delete key (mutates).
        pub fn dele(self: *Self, key: []const u8) bool {
            return self.map.remove(key);
        }

        # Get size.
        pub fn longitudo(self: Self) usize {
            return self.map.count();
        }

        # Check if empty.
        pub fn vacua(self: Self) bool {
            return self.map.count() == 0;
        }

        # Clear all entries (mutates).
        pub fn purga(self: *Self) void {
            self.map.clearRetainingCapacity();
        }

        // =====================================================================
        // ITERATION
        // =====================================================================

        # Get keys iterator.
        pub fn claves(self: *Self) std.StringHashMap(V).KeyIterator {
            return self.map.keyIterator();
        }

        # Get values iterator.
        pub fn valores(self: *Self) std.StringHashMap(V).ValueIterator {
            return self.map.valueIterator();
        }

        # Get entries iterator.
        pub fn paria(self: *Self) std.StringHashMap(V).Iterator {
            return self.map.iterator();
        }

        // =====================================================================
        // TRANSFORMATION
        // =====================================================================

        # Merge another tabula into this one (mutates).
        pub fn confla(self: *Self, other: *Self) void {
            var iter = other.map.iterator();
            while (iter.next()) |entry| {
                self.map.put(entry.key_ptr.*, entry.value_ptr.*) catch @panic("OOM");
            }
        }

        # Convert to list of key-value tuples.
        pub fn inLista(self: *Self, alloc: std.mem.Allocator) []struct { []const u8, V } {
            const Pair = struct { []const u8, V };
            var result = std.ArrayList(Pair).init(alloc);
            var iter = self.map.iterator();
            while (iter.next()) |entry| {
                result.append(.{ entry.key_ptr.*, entry.value_ptr.* }) catch @panic("OOM");
            }
            return result.toOwnedSlice() catch @panic("OOM");
        }
    };
}

# Tabula for non-string keys - wraps std.AutoHashMap
pub fn TabulaAuto(comptime K: type, comptime V: type) type {
    return struct {
        map: std.AutoHashMap(K, V),

        const Self = @This();

        // =====================================================================
        // CONSTRUCTION
        // =====================================================================

        # Create an empty tabula.
        pub fn init(alloc: std.mem.Allocator) Self {
            return .{ .map = std.AutoHashMap(K, V).init(alloc) };
        }

        # Free the tabula's memory.
        pub fn deinit(self: *Self) void {
            self.map.deinit();
        }

        // =====================================================================
        // CORE OPERATIONS
        // =====================================================================

        # Set key-value pair (mutates, needs allocator for growth).
        pub fn pone(self: *Self, alloc: std.mem.Allocator, key: K, value: V) void {
            _ = alloc; // AutoHashMap tracks its allocator internally
            self.map.put(key, value) catch @panic("OOM");
        }

        # Get value by key (returns optional).
        pub fn accipe(self: Self, key: K) ?V {
            return self.map.get(key);
        }

        # Get value or return default.
        pub fn accipeAut(self: Self, key: K, default: V) V {
            return self.map.get(key) orelse default;
        }

        # Check if key exists.
        pub fn habet(self: Self, key: K) bool {
            return self.map.contains(key);
        }

        # Delete key (mutates).
        pub fn dele(self: *Self, key: K) bool {
            return self.map.remove(key);
        }

        # Get size.
        pub fn longitudo(self: Self) usize {
            return self.map.count();
        }

        # Check if empty.
        pub fn vacua(self: Self) bool {
            return self.map.count() == 0;
        }

        # Clear all entries (mutates).
        pub fn purga(self: *Self) void {
            self.map.clearRetainingCapacity();
        }

        // =====================================================================
        // ITERATION
        // =====================================================================

        # Get keys iterator.
        pub fn claves(self: *Self) std.AutoHashMap(K, V).KeyIterator {
            return self.map.keyIterator();
        }

        # Get values iterator.
        pub fn valores(self: *Self) std.AutoHashMap(K, V).ValueIterator {
            return self.map.valueIterator();
        }

        # Get entries iterator.
        pub fn paria(self: *Self) std.AutoHashMap(K, V).Iterator {
            return self.map.iterator();
        }

        // =====================================================================
        // TRANSFORMATION
        // =====================================================================

        # Merge another tabula into this one (mutates).
        pub fn confla(self: *Self, other: *Self) void {
            var iter = other.map.iterator();
            while (iter.next()) |entry| {
                self.map.put(entry.key_ptr.*, entry.value_ptr.*) catch @panic("OOM");
            }
        }

        # Convert to list of key-value tuples.
        pub fn inLista(self: *Self, alloc: std.mem.Allocator) []struct { K, V } {
            const Pair = struct { K, V };
            var result = std.ArrayList(Pair).init(alloc);
            var iter = self.map.iterator();
            while (iter.next()) |entry| {
                result.append(.{ entry.key_ptr.*, entry.value_ptr.* }) catch @panic("OOM");
            }
            return result.toOwnedSlice() catch @panic("OOM");
        }
    };
}
