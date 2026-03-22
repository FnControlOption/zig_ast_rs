const std = @import("std");
const allocator = std.heap.c_allocator;
const assert = std.debug.assert;
const Ast = std.zig.Ast;
const Token = std.zig.Token;

fn ExternInt(comptime T: type) type {
    const info = @typeInfo(T).int;
    const signedness = info.signedness;
    const bits = switch (info.bits) {
        0 => 0,
        else => |bits| std.math.ceilPowerOfTwoAssert(u16, @max(bits, 8)),
    };
    return @Type(.{ .int = .{ .signedness = signedness, .bits = bits } });
}

pub fn ExternEnum(comptime T: type) type {
    var info = @typeInfo(T);
    info.@"enum".tag_type = ExternInt(info.@"enum".tag_type);
    info.@"enum".decls = &.{};
    return @Type(info);
}

const TokenTag = ExternEnum(Token.Tag);
const NodeTag = ExternEnum(Ast.Node.Tag);

const TokenIndex = u32;
const OptionalTokenIndex = u32;
const NodeIndex = u32;
const OptionalNodeIndex = u32;
const ExtraIndex = u32;
const For = u32;

comptime {
    assert(Ast.TokenIndex == TokenIndex);
    assert(@bitSizeOf(Ast.OptionalTokenIndex) == @bitSizeOf(OptionalTokenIndex));
    assert(@bitSizeOf(Ast.Node.Index) == @bitSizeOf(NodeIndex));
    assert(@bitSizeOf(Ast.Node.OptionalIndex) == @bitSizeOf(OptionalNodeIndex));
    assert(@bitSizeOf(Ast.ExtraIndex) == @bitSizeOf(ExtraIndex));
    assert(@bitSizeOf(Ast.Node.For) == @bitSizeOf(For));
}

const SubRange = extern struct {
    start: ExtraIndex,
    end: ExtraIndex,
};

export fn zig_ast_parse(source: [*:0]const u8) ?*Ast {
    const tree = allocator.create(Ast) catch |err| switch (err) {
        error.OutOfMemory => return null,
    };
    const slice = std.mem.sliceTo(source, 0);
    tree.* = Ast.parse(allocator, slice, .zig) catch |err| switch (err) {
        error.OutOfMemory => return null,
    };
    return tree;
}

export fn zig_ast_deinit(tree: *Ast) void {
    tree.deinit(allocator);
    allocator.destroy(tree);
}

export fn zig_ast_source(tree: *const Ast) [*:0]const u8 {
    return tree.source.ptr;
}

export fn zig_ast_token_count(tree: *const Ast) u32 {
    return @intCast(tree.tokens.len);
}

export fn zig_ast_token_tag(tree: *const Ast, index: TokenIndex) TokenTag {
    return @enumFromInt(@intFromEnum(tree.tokenTag(index)));
}

export fn zig_ast_node_count(tree: *const Ast) u32 {
    return @intCast(tree.nodes.len);
}

export fn zig_ast_node_tag(tree: *const Ast, index: NodeIndex) NodeTag {
    return @enumFromInt(@intFromEnum(tree.nodeTag(@enumFromInt(index))));
}

export fn zig_ast_node_source(tree: *const Ast, index: NodeIndex, len: *usize) [*]const u8 {
    const source = tree.getNodeSource(@enumFromInt(index));
    len.* = source.len;
    return source.ptr;
}

export fn zig_ast_node_main_token(tree: *const Ast, index: NodeIndex) TokenIndex {
    return tree.nodeMainToken(@enumFromInt(index));
}

export fn zig_ast_first_token(tree: *const Ast, index: NodeIndex) TokenIndex {
    return tree.firstToken(@enumFromInt(index));
}

export fn zig_ast_last_token(tree: *const Ast, index: NodeIndex) TokenIndex {
    return tree.lastToken(@enumFromInt(index));
}

export fn zig_ast_token_slice(tree: *const Ast, index: TokenIndex, len: *usize) [*]const u8 {
    const slice = tree.tokenSlice(index);
    len.* = slice.len;
    return slice.ptr;
}

export fn zig_ast_extra_data(tree: *const Ast) [*]const u32 {
    return tree.extra_data.ptr;
}

export fn zig_ast_extra_data_len(tree: *const Ast) u32 {
    return @intCast(tree.extra_data.len);
}

export fn zig_ast_node_data_node(tree: *const Ast, index: NodeIndex) NodeIndex {
    const node_data = tree.nodeData(@enumFromInt(index));
    return @intFromEnum(node_data.node);
}

export fn zig_ast_node_data_opt_node(tree: *const Ast, index: NodeIndex) OptionalNodeIndex {
    const node_data = tree.nodeData(@enumFromInt(index));
    return @intFromEnum(node_data.opt_node);
}

export fn zig_ast_node_data_token(tree: *const Ast, index: NodeIndex) TokenIndex {
    const node_data = tree.nodeData(@enumFromInt(index));
    return node_data.token;
}

export fn zig_ast_node_data_extra_range(tree: *const Ast, index: NodeIndex) SubRange {
    const node_data = tree.nodeData(@enumFromInt(index));
    const range = node_data.extra_range;
    return .{
        .start = @intFromEnum(range.start),
        .end = @intFromEnum(range.end),
    };
}

fn toInt(comptime T: type, value: anytype) T {
    const Value = @TypeOf(value);
    comptime assert(@typeInfo(T) == .int);
    comptime assert(@bitSizeOf(T) == @bitSizeOf(Value));
    return switch (@typeInfo(Value)) {
        .@"enum" => @intFromEnum(value),
        .int => value,
        else => @bitCast(value),
    };
}

fn exportNodeDataPair(
    comptime Lhs: type,
    comptime Rhs: type,
    comptime field_name: []const u8,
) void {
    const T = extern struct {
        lhs: Lhs,
        rhs: Rhs,

        fn zig_ast_node_data(tree: *const Ast, index: NodeIndex) callconv(.c) @This() {
            const node_data = tree.nodeData(@enumFromInt(index));
            const lhs, const rhs = @field(node_data, field_name);
            return .{ .lhs = toInt(Lhs, lhs), .rhs = toInt(Rhs, rhs) };
        }
    };

    @export(&T.zig_ast_node_data, .{ .name = "zig_ast_node_data_" ++ field_name });
}

comptime {
    exportNodeDataPair(NodeIndex, NodeIndex, "node_and_node");
    exportNodeDataPair(OptionalNodeIndex, OptionalNodeIndex, "opt_node_and_opt_node");
    exportNodeDataPair(NodeIndex, OptionalNodeIndex, "node_and_opt_node");
    exportNodeDataPair(OptionalNodeIndex, NodeIndex, "opt_node_and_node");
    exportNodeDataPair(NodeIndex, ExtraIndex, "node_and_extra");
    exportNodeDataPair(ExtraIndex, NodeIndex, "extra_and_node");
    exportNodeDataPair(ExtraIndex, OptionalNodeIndex, "extra_and_opt_node");
    exportNodeDataPair(NodeIndex, TokenIndex, "node_and_token");
    exportNodeDataPair(TokenIndex, NodeIndex, "token_and_node");
    exportNodeDataPair(TokenIndex, TokenIndex, "token_and_token");
    exportNodeDataPair(OptionalNodeIndex, TokenIndex, "opt_node_and_token");
    exportNodeDataPair(OptionalTokenIndex, NodeIndex, "opt_token_and_node");
    exportNodeDataPair(OptionalTokenIndex, OptionalNodeIndex, "opt_token_and_opt_node");
    exportNodeDataPair(OptionalTokenIndex, OptionalTokenIndex, "opt_token_and_opt_token");
    exportNodeDataPair(ExtraIndex, For, "for");
}

pub fn ExternStruct(comptime T: type) type {
    const info = @typeInfo(T).@"struct";
    var new_info = info;
    new_info.layout = .@"extern";
    new_info.decls = &.{};
    new_info.fields = &.{};
    for (info.fields) |field| {
        switch (field.type) {
            []const Ast.Node.Index => {
                var ptr_field = field;
                ptr_field.name = field.name ++ "_ptr";
                ptr_field.type = [*]const ExternEnum(Ast.Node.Index);

                var len_field = field;
                len_field.name = field.name ++ "_len";
                len_field.type = usize;

                new_info.fields = new_info.fields ++ .{ ptr_field, len_field };
            },
            else => {
                var new_field = field;
                new_field.type = ExternType(field.type);
                new_info.fields = new_info.fields ++ .{new_field};
            },
        }
    }
    return @Type(.{ .@"struct" = new_info });
}

fn toExternStruct(value: anytype) ExternStruct(@TypeOf(value)) {
    const T = @TypeOf(value);
    const info = @typeInfo(T).@"struct";
    var new_value: ExternStruct(T) = undefined;
    inline for (info.fields) |field| {
        switch (field.type) {
            []const Ast.Node.Index => {
                @field(new_value, field.name ++ "_ptr") = @ptrCast(@field(value, field.name).ptr);
                @field(new_value, field.name ++ "_len") = @field(value, field.name).len;
            },
            else => {
                @field(new_value, field.name) = toExtern(@field(value, field.name));
            },
        }
    }
    return new_value;
}

pub fn ExternType(comptime T: type) type {
    @setEvalBranchQuota(2000);
    return switch (@typeInfo(T)) {
        // pointer (i.e. slice) types are only supported as struct fields
        .pointer => comptime unreachable,
        .@"struct" => ExternStruct(T),
        .@"enum" => ExternEnum(T),
        .optional => |info| switch (info.child) {
            Ast.TokenIndex => ExternEnum(Ast.OptionalTokenIndex),
            Ast.Node.Index => ExternEnum(Ast.Node.OptionalIndex),
            else => comptime unreachable,
        },
        else => switch (T) {
            bool => bool,
            Ast.TokenIndex => TokenIndex,
            else => comptime unreachable,
        },
    };
}

fn toExtern(value: anytype) ExternType(@TypeOf(value)) {
    const T = @TypeOf(value);
    return switch (@typeInfo(T)) {
        .pointer => comptime unreachable,
        .@"struct" => toExternStruct(value),
        .@"enum" => @enumFromInt(@intFromEnum(value)),
        .optional => |info| switch (info.child) {
            Ast.TokenIndex => if (value) |v| @enumFromInt(v) else .none,
            Ast.Node.Index => if (value) |v| @enumFromInt(@intFromEnum(v)) else .none,
            else => comptime unreachable,
        },
        else => switch (T) {
            bool => value,
            Ast.TokenIndex => value,
            else => comptime unreachable,
        },
    };
}

fn exportFull(comptime name: []const u8) void {
    @setEvalBranchQuota(3000);
    const T = @field(Ast.full, name);
    const ExternT = ExternStruct(T);
    const fullT = @field(Ast, "full" ++ name);
    const FullT = @TypeOf(fullT);
    const fn_info = @typeInfo(FullT).@"fn";
    assert(fn_info.return_type == ?T);

    var param_types: [fn_info.params.len]type = undefined;
    for (&param_types, fn_info.params) |*param_type, param|
        param_type.* = param.type.?;

    const Export = if (std.mem.eql(type, &param_types, &.{ Ast, Ast.Node.Index })) struct {
        fn zig_ast_full(tree: *const Ast, index: NodeIndex, extern_t: *ExternT) callconv(.c) bool {
            const t = fullT(tree.*, @enumFromInt(index)) orelse return false;
            extern_t.* = toExternStruct(t);
            return true;
        }
    } else if (std.mem.eql(type, &param_types, &.{ Ast, *[1]Ast.Node.Index, Ast.Node.Index })) struct {
        fn zig_ast_full(tree: *const Ast, buffer: *[1]NodeIndex, index: NodeIndex, extern_t: *ExternT) callconv(.c) bool {
            const t = fullT(tree.*, @ptrCast(buffer), @enumFromInt(index)) orelse return false;
            extern_t.* = toExternStruct(t);
            return true;
        }
    } else if (std.mem.eql(type, &param_types, &.{ Ast, *[2]Ast.Node.Index, Ast.Node.Index })) struct {
        fn zig_ast_full(tree: *const Ast, buffer: *[2]NodeIndex, index: NodeIndex, extern_t: *ExternT) callconv(.c) bool {
            const t = fullT(tree.*, @ptrCast(buffer), @enumFromInt(index)) orelse return false;
            extern_t.* = toExternStruct(t);
            return true;
        }
    } else comptime unreachable;

    @export(&Export.zig_ast_full, .{ .name = "zig_ast_full_" ++ comptimeCamelToSnakeCase(name) });
}

comptime {
    assert(@intFromEnum(Ast.OptionalTokenIndex.none) == std.math.maxInt(OptionalTokenIndex));
    assert(@intFromEnum(Ast.Node.OptionalIndex.none) == std.math.maxInt(OptionalNodeIndex));
    for (std.meta.declarations(Ast.full)) |decl| {
        if (std.mem.eql(u8, decl.name, "AssignDestructure")) continue;
        if (std.mem.eql(u8, decl.name, "AsmLegacy")) continue;
        exportFull(decl.name);
    }
}

export fn zig_ast_assign_destructure(tree: *const Ast, index: NodeIndex) ExternStruct(Ast.full.AssignDestructure) {
    return toExternStruct(tree.assignDestructure(@enumFromInt(index)));
}

export fn zig_ast_builtin_call_params(tree: *const Ast, buffer: *[2]NodeIndex, index: NodeIndex, count: *usize) ?[*]const NodeIndex {
    if (tree.builtinCallParams(@ptrCast(buffer), @enumFromInt(index))) |params| {
        count.* = params.len;
        return @ptrCast(params.ptr);
    } else {
        return null;
    }
}

export fn zig_ast_block_statements(tree: *const Ast, buffer: *[2]NodeIndex, index: NodeIndex, count: *usize) ?[*]const NodeIndex {
    if (tree.blockStatements(@ptrCast(buffer), @enumFromInt(index))) |statements| {
        count.* = statements.len;
        return @ptrCast(statements.ptr);
    } else {
        return null;
    }
}

pub fn comptimeCamelToSnakeCase(comptime name: []const u8) []const u8 {
    return comptime blk: {
        var result: []const u8 = "";
        var start: usize = 0;
        for (name, 0..) |c, i| {
            result = result ++ name[start..i];
            if (std.ascii.isUpper(c) and i > 0)
                result = result ++ "_";
            result = result ++ .{std.ascii.toLower(c)};
            start = i + 1;
        }
        break :blk result;
    };
}

test comptimeCamelToSnakeCase {
    try std.testing.expectEqualStrings("foo", comptimeCamelToSnakeCase("foo"));
    try std.testing.expectEqualStrings("foo", comptimeCamelToSnakeCase("Foo"));
    try std.testing.expectEqualStrings("foo_bar", comptimeCamelToSnakeCase("fooBar"));
    try std.testing.expectEqualStrings("foo_bar", comptimeCamelToSnakeCase("FooBar"));
    try std.testing.expectEqualStrings("foo_bar_baz", comptimeCamelToSnakeCase("fooBarBaz"));
    try std.testing.expectEqualStrings("foo_bar_baz", comptimeCamelToSnakeCase("FooBarBaz"));
}

export fn zig_ast_builtin_fn_tag(name: [*]const u8, len: usize, tag: *ExternEnum(std.zig.BuiltinFn.Tag)) bool {
    if (std.zig.BuiltinFn.list.get(name[0..len])) |builtin| {
        tag.* = toExtern(builtin.tag);
        return true;
    }
    return false;
}
