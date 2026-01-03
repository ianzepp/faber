// Lista - Dynamic array wrapper for Faber
// Wraps std.ArrayList with Latin method names
//
// WHY: No stored allocator. Methods that need allocation accept it as parameter.
// This ensures cura...fit blocks properly thread the current allocator.

const std = @import("std");

# Lista(T) is a dynamic array with Latin-named methods.
# Wraps std.ArrayList(T). Allocator passed per-method, not stored.
pub fn Lista(comptime T: type) type {
    return struct {
        items: std.ArrayList(T),

        const Self = @This();

        // =====================================================================
        // CONSTRUCTION
        // =====================================================================

        # Create an empty lista.
        pub fn init(alloc: std.mem.Allocator) Self {
            return .{ .items = std.ArrayList(T).init(alloc) };
        }

        # Create a lista from a slice.
        pub fn fromItems(alloc: std.mem.Allocator, data: []const T) Self {
            var self = init(alloc);
            self.items.appendSlice(data) catch @panic("OOM");
            return self;
        }

        # Free the lista's memory.
        pub fn deinit(self: *Self) void {
            self.items.deinit();
        }

        # Clone the lista (deep copy).
        pub fn clone(self: Self, alloc: std.mem.Allocator) Self {
            var result = init(alloc);
            result.items.appendSlice(self.items.items) catch @panic("OOM");
            return result;
        }

        // =====================================================================
        // ADDING ELEMENTS
        // =====================================================================

        # Add element to end (mutates).
        pub fn adde(self: *Self, alloc: std.mem.Allocator, value: T) void {
            _ = alloc; // ArrayList tracks its allocator internally
            self.items.append(value) catch @panic("OOM");
        }

        # Add element to end, returns new lista (immutable).
        pub fn addita(self: Self, alloc: std.mem.Allocator, value: T) Self {
            var result = self.clone(alloc);
            result.items.append(value) catch @panic("OOM");
            return result;
        }

        # Add element to start (mutates).
        pub fn praepone(self: *Self, alloc: std.mem.Allocator, value: T) void {
            _ = alloc;
            self.items.insert(0, value) catch @panic("OOM");
        }

        # Add element to start, returns new lista (immutable).
        pub fn praeposita(self: Self, alloc: std.mem.Allocator, value: T) Self {
            var result = self.clone(alloc);
            result.items.insert(0, value) catch @panic("OOM");
            return result;
        }

        // =====================================================================
        // REMOVING ELEMENTS
        // =====================================================================

        # Remove and return last element (mutates).
        pub fn remove(self: *Self) ?T {
            return self.items.popOrNull();
        }

        # Remove last element, returns new lista (immutable).
        pub fn remota(self: Self, alloc: std.mem.Allocator) Self {
            if (self.items.items.len == 0) return self.clone(alloc);
            var result = init(alloc);
            result.items.appendSlice(self.items.items[0 .. self.items.items.len - 1]) catch @panic("OOM");
            return result;
        }

        # Remove and return first element (mutates).
        pub fn decapita(self: *Self) ?T {
            if (self.items.items.len == 0) return null;
            return self.items.orderedRemove(0);
        }

        # Remove first element, returns new lista (immutable).
        pub fn decapitata(self: Self, alloc: std.mem.Allocator) Self {
            if (self.items.items.len == 0) return self.clone(alloc);
            var result = init(alloc);
            result.items.appendSlice(self.items.items[1..]) catch @panic("OOM");
            return result;
        }

        # Clear all elements (mutates).
        pub fn purga(self: *Self) void {
            self.items.clearRetainingCapacity();
        }

        // =====================================================================
        // ACCESSING ELEMENTS
        // =====================================================================

        # Get first element.
        pub fn primus(self: Self) ?T {
            if (self.items.items.len == 0) return null;
            return self.items.items[0];
        }

        # Get last element.
        pub fn ultimus(self: Self) ?T {
            if (self.items.items.len == 0) return null;
            return self.items.items[self.items.items.len - 1];
        }

        # Get element at index.
        pub fn accipe(self: Self, index: usize) ?T {
            if (index >= self.items.items.len) return null;
            return self.items.items[index];
        }

        # Get raw slice for iteration.
        pub fn toSlice(self: Self) []T {
            return self.items.items;
        }

        // =====================================================================
        // PROPERTIES
        // =====================================================================

        # Get length.
        pub fn longitudo(self: Self) usize {
            return self.items.items.len;
        }

        # Check if empty.
        pub fn vacua(self: Self) bool {
            return self.items.items.len == 0;
        }

        // =====================================================================
        // SEARCHING
        // =====================================================================

        # Check if contains element.
        pub fn continet(self: Self, value: T) bool {
            return std.mem.indexOfScalar(T, self.items.items, value) != null;
        }

        # Find index of element.
        pub fn indiceDe(self: Self, value: T) ?usize {
            return std.mem.indexOfScalar(T, self.items.items, value);
        }

        // =====================================================================
        // PREDICATE METHODS
        // =====================================================================

        # Check if all elements match predicate.
        pub fn omnes(self: Self, predicate: *const fn (T) bool) bool {
            for (self.items.items) |v| {
                if (!predicate(v)) return false;
            }
            return true;
        }

        # Check if any element matches predicate.
        pub fn aliquis(self: Self, predicate: *const fn (T) bool) bool {
            for (self.items.items) |v| {
                if (predicate(v)) return true;
            }
            return false;
        }

        # Find first element matching predicate.
        pub fn inveni(self: Self, predicate: *const fn (T) bool) ?T {
            for (self.items.items) |v| {
                if (predicate(v)) return v;
            }
            return null;
        }

        # Find index of first element matching predicate.
        pub fn inveniIndicem(self: Self, predicate: *const fn (T) bool) ?usize {
            for (self.items.items, 0..) |v, i| {
                if (predicate(v)) return i;
            }
            return null;
        }

        // =====================================================================
        // FUNCTIONAL METHODS (allocating, return new lista)
        // =====================================================================

        # Filter elements matching predicate.
        pub fn filtrata(self: Self, alloc: std.mem.Allocator, predicate: *const fn (T) bool) Self {
            var result = init(alloc);
            for (self.items.items) |v| {
                if (predicate(v)) result.items.append(v) catch @panic("OOM");
            }
            return result;
        }

        # Map elements through transform function.
        # Returns Lista of same type (T -> T transform).
        pub fn mappata(self: Self, alloc: std.mem.Allocator, transform: *const fn (T) T) Self {
            var result = init(alloc);
            for (self.items.items) |v| {
                result.items.append(transform(v)) catch @panic("OOM");
            }
            return result;
        }

        # Reduce to single value.
        pub fn reducta(self: Self, reducer: *const fn (T, T) T, initial: T) T {
            var acc = initial;
            for (self.items.items) |v| {
                acc = reducer(acc, v);
            }
            return acc;
        }

        # Reverse (returns new lista).
        pub fn inversa(self: Self, alloc: std.mem.Allocator) Self {
            var result = init(alloc);
            var i: usize = self.items.items.len;
            while (i > 0) {
                i -= 1;
                result.items.append(self.items.items[i]) catch @panic("OOM");
            }
            return result;
        }

        # Sort (returns new lista).
        pub fn ordinata(self: Self, alloc: std.mem.Allocator) Self {
            const result = self.clone(alloc);
            std.mem.sort(T, result.items.items, {}, std.sort.asc(T));
            return result;
        }

        # Sort with custom comparator (returns new lista).
        pub fn ordinataCum(self: Self, alloc: std.mem.Allocator, comptime lessThan: fn (void, T, T) bool) Self {
            const result = self.clone(alloc);
            std.mem.sort(T, result.items.items, {}, lessThan);
            return result;
        }

        # Slice - take elements from start to end.
        pub fn sectio(self: Self, alloc: std.mem.Allocator, start: usize, end: usize) Self {
            var result = init(alloc);
            const s = @min(start, self.items.items.len);
            const e = @min(end, self.items.items.len);
            if (s < e) {
                result.items.appendSlice(self.items.items[s..e]) catch @panic("OOM");
            }
            return result;
        }

        # Take first n elements.
        pub fn prima(self: Self, alloc: std.mem.Allocator, n: usize) Self {
            const count = @min(n, self.items.items.len);
            var result = init(alloc);
            result.items.appendSlice(self.items.items[0..count]) catch @panic("OOM");
            return result;
        }

        # Take last n elements.
        pub fn ultima(self: Self, alloc: std.mem.Allocator, n: usize) Self {
            const count = @min(n, self.items.items.len);
            const start = self.items.items.len - count;
            var result = init(alloc);
            result.items.appendSlice(self.items.items[start..]) catch @panic("OOM");
            return result;
        }

        # Skip first n elements.
        pub fn omitte(self: Self, alloc: std.mem.Allocator, n: usize) Self {
            const skip = @min(n, self.items.items.len);
            var result = init(alloc);
            result.items.appendSlice(self.items.items[skip..]) catch @panic("OOM");
            return result;
        }

        // =====================================================================
        // MUTATING OPERATIONS
        // =====================================================================

        # Sort in place.
        pub fn ordina(self: *Self) void {
            std.mem.sort(T, self.items.items, {}, std.sort.asc(T));
        }

        # Sort in place with custom comparator.
        pub fn ordinaCum(self: *Self, comptime lessThan: fn (void, T, T) bool) void {
            std.mem.sort(T, self.items.items, {}, lessThan);
        }

        # Reverse in place.
        pub fn inverte(self: *Self) void {
            std.mem.reverse(T, self.items.items);
        }

        // =====================================================================
        // ITERATION
        // =====================================================================

        # Execute callback for each element.
        pub fn perambula(self: Self, callback: *const fn (T) void) void {
            for (self.items.items) |v| {
                callback(v);
            }
        }

        // =====================================================================
        // AGGREGATION (numeric types only - caller must ensure T is numeric)
        // =====================================================================

        # Sum of numeric elements.
        # Only valid for integer types.
        pub fn summa(self: Self) T {
            var sum: T = 0;
            for (self.items.items) |v| {
                sum += v;
            }
            return sum;
        }

        # Average of numeric elements.
        # Returns f64 for precision.
        pub fn medium(self: Self) f64 {
            if (self.items.items.len == 0) return 0.0;
            var sum: T = 0;
            for (self.items.items) |v| {
                sum += v;
            }
            return @as(f64, @floatFromInt(sum)) / @as(f64, @floatFromInt(self.items.items.len));
        }

        # Minimum value.
        pub fn minimus(self: Self) ?T {
            return std.mem.min(T, self.items.items);
        }

        # Maximum value.
        pub fn maximus(self: Self) ?T {
            return std.mem.max(T, self.items.items);
        }

        # Count elements matching predicate.
        pub fn numera(self: Self, predicate: ?*const fn (T) bool) usize {
            if (predicate) |pred| {
                var count: usize = 0;
                for (self.items.items) |v| {
                    if (pred(v)) count += 1;
                }
                return count;
            }
            return self.items.items.len;
        }
    };
}
