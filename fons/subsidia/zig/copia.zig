// Copia - HashSet wrapper for Faber
// Wraps std.AutoHashMap(T, void) as a set type
//
// WHY: No stored allocator. Methods that need allocation accept it as parameter.
// This ensures cura...fit blocks properly thread the current allocator.
//
// WHY: Zig has no native Set type. We use HashMap with void values.
// For string elements, use CopiaTextus which wraps StringHashMap(void).

const std = @import("std");

# Copia(T) is a hash set with Latin-named methods.
# For string elements, use CopiaTextus which wraps StringHashMap(void).
# For other types, use CopiaAuto(T) which wraps AutoHashMap(T, void).
pub fn Copia(comptime T: type) type {
    if (T == []const u8) {
        return CopiaTextus;
    }
    return CopiaAuto(T);
}

# Copia for string elements - wraps std.StringHashMap(void)
pub const CopiaTextus = struct {
    map: std.StringHashMap(void),

    const Self = @This();

    // =====================================================================
    // CONSTRUCTION
    // =====================================================================

    # Create an empty copia.
    pub fn init(alloc: std.mem.Allocator) Self {
        return .{ .map = std.StringHashMap(void).init(alloc) };
    }

    # Free the copia's memory.
    pub fn deinit(self: *Self) void {
        self.map.deinit();
    }

    // =====================================================================
    // CORE OPERATIONS
    // =====================================================================

    # Add element (mutates, needs allocator for growth).
    pub fn adde(self: *Self, alloc: std.mem.Allocator, value: []const u8) void {
        _ = alloc; // StringHashMap tracks its allocator internally
        self.map.put(value, {}) catch @panic("OOM");
    }

    # Check if element exists.
    pub fn habet(self: Self, value: []const u8) bool {
        return self.map.contains(value);
    }

    # Delete element (mutates).
    pub fn dele(self: *Self, value: []const u8) bool {
        return self.map.remove(value);
    }

    # Get size.
    pub fn longitudo(self: Self) usize {
        return self.map.count();
    }

    # Check if empty.
    pub fn vacua(self: Self) bool {
        return self.map.count() == 0;
    }

    # Clear all elements (mutates).
    pub fn purga(self: *Self) void {
        self.map.clearRetainingCapacity();
    }

    // =====================================================================
    // ITERATION
    // =====================================================================

    # Get values (keys) iterator.
    pub fn valores(self: *Self) std.StringHashMap(void).KeyIterator {
        return self.map.keyIterator();
    }
};

# Copia for non-string elements - wraps std.AutoHashMap(T, void)
pub fn CopiaAuto(comptime T: type) type {
    return struct {
        map: std.AutoHashMap(T, void),

        const Self = @This();

        // =====================================================================
        // CONSTRUCTION
        // =====================================================================

        # Create an empty copia.
        pub fn init(alloc: std.mem.Allocator) Self {
            return .{ .map = std.AutoHashMap(T, void).init(alloc) };
        }

        # Free the copia's memory.
        pub fn deinit(self: *Self) void {
            self.map.deinit();
        }

        // =====================================================================
        // CORE OPERATIONS
        // =====================================================================

        # Add element (mutates, needs allocator for growth).
        pub fn adde(self: *Self, alloc: std.mem.Allocator, value: T) void {
            _ = alloc; // AutoHashMap tracks its allocator internally
            self.map.put(value, {}) catch @panic("OOM");
        }

        # Check if element exists.
        pub fn habet(self: Self, value: T) bool {
            return self.map.contains(value);
        }

        # Delete element (mutates).
        pub fn dele(self: *Self, value: T) bool {
            return self.map.remove(value);
        }

        # Get size.
        pub fn longitudo(self: Self) usize {
            return self.map.count();
        }

        # Check if empty.
        pub fn vacua(self: Self) bool {
            return self.map.count() == 0;
        }

        # Clear all elements (mutates).
        pub fn purga(self: *Self) void {
            self.map.clearRetainingCapacity();
        }

        // =====================================================================
        // ITERATION
        // =====================================================================

        # Get values (keys) iterator.
        pub fn valores(self: *Self) std.AutoHashMap(T, void).KeyIterator {
            return self.map.keyIterator();
        }
    };
}
