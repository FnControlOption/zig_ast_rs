use std::fmt::Debug;
use std::marker::{PhantomData, PhantomPinned};

pub use extra_data::*;
pub use node_data::*;

mod enums;
pub use enums::*;

pub mod full;

// https://doc.rust-lang.org/nomicon/ffi.html#representing-opaque-structs
#[repr(C)]
pub struct Ast {
    _data: (),
    _marker: PhantomData<(*mut u8, PhantomPinned)>,
}

macro_rules! ast {
    ($($rs:ident / $c:ident($($param:ident: $type:ty),*)$( -> $return:ty)?;)+) => {
        impl Ast {
            $(
                pub unsafe fn $rs($($param: $type),*)$( -> $return)? {
                    unsafe { $c($($param),*) }
                }
            )+
        }

        unsafe extern "C" {
            $(
                fn $c($($param: $type),*)$( -> $return)?;
            )+
        }
    };
}

ast! {
    parse / zig_ast_parse(source: *const u8) -> *mut Ast;
    deinit / zig_ast_deinit(tree: *mut Ast);

    source / zig_ast_source(tree: *const Ast) -> *const u8;
    token_count / zig_ast_token_count(tree: *const Ast) -> u32;
    token_tag / zig_ast_token_tag(tree: *const Ast, index: TokenIndex) -> TokenTag;
    token_start / zig_ast_token_start(tree: *const Ast, index: TokenIndex) -> u32;
    node_count / zig_ast_node_count(tree: *const Ast) -> u32;
    node_tag / zig_ast_node_tag(tree: *const Ast, index: NodeIndex) -> NodeTag;
    node_source / zig_ast_node_source(tree: *const Ast, index: NodeIndex, len: *mut usize) -> *const u8;
    node_main_token / zig_ast_node_main_token(tree: *const Ast, index: NodeIndex) -> TokenIndex;
    first_token / zig_ast_first_token(tree: *const Ast, index: NodeIndex) -> TokenIndex;
    last_token / zig_ast_last_token(tree: *const Ast, index: NodeIndex) -> TokenIndex;
    token_slice / zig_ast_token_slice(tree: *const Ast, index: TokenIndex, len: *mut usize) -> *const u8;
    token_length / zig_ast_token_length(tree: *const Ast, index: TokenIndex) -> u32;
    extra_data / zig_ast_extra_data(tree: *const Ast) -> *const u32;
    extra_data_len / zig_ast_extra_data_len(tree: *const Ast) -> u32;
    builtin_call_params / zig_ast_builtin_call_params(tree: *const Ast, buffer: *mut [NodeIndex; 2], index: NodeIndex, count: *mut usize) -> *const NodeIndex;
    block_statements / zig_ast_block_statements(tree: *const Ast, buffer: *mut [NodeIndex; 2], index: NodeIndex, count: *mut usize) -> *const NodeIndex;

    builtin_fn_tag / zig_ast_builtin_fn_tag(name: *const u8, len: usize, tag: *mut BuiltinFnTag) -> bool;
    parse_string_literal / zig_ast_parse_string_literal(ptr: *const u8, len: *mut usize) -> *mut u8;
    free_string / zig_ast_free_string(ptr: *mut u8, len: usize);
}

pub trait Full<'ast, const N: usize>: Sized {
    unsafe fn from(tree: *const Ast, buffer: *mut [NodeIndex; N], index: NodeIndex)
    -> Option<Self>;
}

impl<'ast> Full<'ast, 0> for full::AssignDestructure<'ast> {
    unsafe fn from(
        tree: *const Ast,
        _buffer: *mut [NodeIndex; 0],
        index: NodeIndex,
    ) -> Option<Self> {
        let node_tag = unsafe { Ast::node_tag(tree, index) };
        match node_tag {
            NodeTag::AssignDestructure => Some(unsafe { zig_ast_assign_destructure(tree, index) }),
            _ => None,
        }
    }
}

unsafe extern "C" {
    fn zig_ast_assign_destructure<'ast>(
        tree: *const Ast,
        index: NodeIndex,
    ) -> full::AssignDestructure<'ast>;
}

unsafe fn slice_from_raw_parts<'a>(ptr: *const NodeIndex, len: usize) -> &'a [NodeIndex] {
    match len {
        0 => &[],
        _ => unsafe { std::slice::from_raw_parts(ptr, len) },
    }
}

impl<'ast> full::AssignDestructureComponents<'ast> {
    pub fn variables(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.variables_ptr, self.variables_len) }
    }
}

impl<'ast> full::ForComponents<'ast> {
    pub fn inputs(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.inputs_ptr, self.inputs_len) }
    }
}

impl<'ast> full::FnProtoComponents<'ast> {
    pub fn params(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.params_ptr, self.params_len) }
    }
}

impl<'ast> full::StructInitComponents<'ast> {
    pub fn fields(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.fields_ptr, self.fields_len) }
    }
}

impl<'ast> full::ArrayInitComponents<'ast> {
    pub fn elements(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.elements_ptr, self.elements_len) }
    }
}

impl<'ast> full::ContainerDeclComponents<'ast> {
    pub fn members(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.members_ptr, self.members_len) }
    }
}

impl<'ast> full::SwitchComponents<'ast> {
    pub fn cases(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.cases_ptr, self.cases_len) }
    }
}

impl<'ast> full::SwitchCaseComponents<'ast> {
    pub fn values(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.values_ptr, self.values_len) }
    }
}

impl<'ast> full::Asm<'ast> {
    pub fn outputs(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.outputs_ptr, self.outputs_len) }
    }

    pub fn inputs(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.inputs_ptr, self.inputs_len) }
    }
}

impl<'ast> full::AsmComponents<'ast> {
    pub fn items(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.items_ptr, self.items_len) }
    }
}

impl<'ast> full::CallComponents<'ast> {
    pub fn params(&'ast self) -> &'ast [NodeIndex] {
        unsafe { slice_from_raw_parts(self.params_ptr, self.params_len) }
    }
}

macro_rules! index_newtype {
    ($Index:ident, $Iterator:ident) => {
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $Index(pub u32);

        impl From<u32> for $Index {
            fn from(value: u32) -> Self {
                Self(value)
            }
        }

        pub struct $Iterator {
            index: u32,
            end: u32,
        }

        impl $Iterator {
            pub fn from_range(start: $Index, end: $Index) -> Self {
                let index = start.0;
                let end = end.0;
                Self { index, end }
            }
        }

        impl Iterator for $Iterator {
            type Item = $Index;

            fn next(&mut self) -> Option<$Index> {
                if self.index < self.end {
                    let result = self.index;
                    self.index += 1;
                    Some($Index(result))
                } else {
                    None
                }
            }
        }
    };

    ($Index:ident, $Iterator:ident, $Optional:ident) => {
        index_newtype!($Index, $Iterator);

        #[repr(transparent)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $Optional(pub u32);

        impl $Optional {
            pub const NONE: Self = Self(u32::MAX);

            pub fn is_none(self) -> bool {
                self == Self::NONE
            }

            pub fn to_option(self) -> Option<$Index> {
                if self == Self::NONE {
                    None
                } else {
                    Some($Index(self.0))
                }
            }
        }

        impl Debug for $Optional {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if *self == Self::NONE {
                    write!(f, concat!(stringify!($Optional), "::NONE"))
                } else {
                    f.debug_tuple(stringify!($Optional)).field(&self.0).finish()
                }
            }
        }

        impl From<u32> for $Optional {
            fn from(value: u32) -> Self {
                Self(value)
            }
        }

        impl From<$Index> for $Optional {
            fn from(value: $Index) -> Self {
                Self(value.0)
            }
        }

        impl From<Option<$Index>> for $Optional {
            fn from(value: Option<$Index>) -> Self {
                match value {
                    Some(value) => Self(value.0),
                    None => Self::NONE,
                }
            }
        }
    };
}

index_newtype!(TokenIndex, TokenIndexIterator, OptionalTokenIndex);
index_newtype!(NodeIndex, NodeIndexIterator, OptionalNodeIndex);
index_newtype!(ExtraIndex, ExtraIndexIterator);

impl NodeIndex {
    pub const ROOT: Self = Self(0);

    pub fn is_root(self) -> bool {
        self == Self::ROOT
    }
}

impl OptionalNodeIndex {
    pub const ROOT: Self = Self(0);

    pub fn is_root(self) -> bool {
        self == Self::ROOT
    }
}

impl From<SubRange> for ExtraIndexIterator {
    fn from(value: SubRange) -> Self {
        let index = value.start.0;
        let end = value.end.0;
        Self { index, end }
    }
}

pub mod node_data {
    use super::*;

    pub trait NodeData {
        unsafe fn from(tree: *const Ast, index: NodeIndex) -> Self;
    }

    unsafe extern "C" {
        fn zig_ast_node_data_node(tree: *const Ast, index: NodeIndex) -> NodeIndex;
        fn zig_ast_node_data_opt_node(tree: *const Ast, index: NodeIndex) -> OptionalNodeIndex;
        fn zig_ast_node_data_token(tree: *const Ast, index: NodeIndex) -> TokenIndex;
        fn zig_ast_node_data_extra_range(tree: *const Ast, index: NodeIndex) -> SubRange;
    }

    impl NodeData for NodeIndex {
        unsafe fn from(tree: *const Ast, index: NodeIndex) -> Self {
            unsafe { zig_ast_node_data_node(tree, index) }
        }
    }

    impl NodeData for OptionalNodeIndex {
        unsafe fn from(tree: *const Ast, index: NodeIndex) -> Self {
            unsafe { zig_ast_node_data_opt_node(tree, index) }
        }
    }

    impl NodeData for TokenIndex {
        unsafe fn from(tree: *const Ast, index: NodeIndex) -> Self {
            unsafe { zig_ast_node_data_token(tree, index) }
        }
    }

    impl NodeData for SubRange {
        unsafe fn from(tree: *const Ast, index: NodeIndex) -> Self {
            unsafe { zig_ast_node_data_extra_range(tree, index) }
        }
    }

    macro_rules! node_data_pair {
        ($Data:ident = ($Lhs:path, $Rhs:path, $from:ident)) => {
            unsafe extern "C" {
                fn $from(tree: *const Ast, index: NodeIndex) -> $Data;
            }

            #[repr(C)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub struct $Data(pub $Lhs, pub $Rhs);

            impl $Data {
                pub fn into_tuple(self) -> ($Lhs, $Rhs) {
                    let Self(lhs, rhs) = self;
                    (lhs, rhs)
                }
            }

            impl NodeData for $Data {
                unsafe fn from(tree: *const Ast, index: NodeIndex) -> Self {
                    unsafe { $from(tree, index) }
                }
            }

            impl From<($Lhs, $Rhs)> for $Data {
                fn from((lhs, rhs): ($Lhs, $Rhs)) -> Self {
                    Self(lhs, rhs)
                }
            }
        };
    }

    macro_rules! node_data_pairs {
        ($($Data:ident = ($Lhs:path, $Rhs:path, $from:ident),)+) => {
            $(node_data_pair!($Data =($Lhs, $Rhs, $from));)+
        };
    }

    node_data_pairs! {
        NodeAndNode = (NodeIndex, NodeIndex, zig_ast_node_data_node_and_node),
        OptNodeAndOptNode = (OptionalNodeIndex, OptionalNodeIndex, zig_ast_node_data_opt_node_and_opt_node),
        NodeAndOptNode = (NodeIndex, OptionalNodeIndex, zig_ast_node_data_node_and_opt_node),
        OptNodeAndNode = (OptionalNodeIndex, NodeIndex, zig_ast_node_data_opt_node_and_node),
        NodeAndExtra = (NodeIndex, ExtraIndex, zig_ast_node_data_node_and_extra),
        ExtraAndNode = (ExtraIndex, NodeIndex, zig_ast_node_data_extra_and_node),
        ExtraAndOptNode = (ExtraIndex, OptionalNodeIndex, zig_ast_node_data_extra_and_opt_node),
        NodeAndToken = (NodeIndex, TokenIndex, zig_ast_node_data_node_and_token),
        TokenAndNode = (TokenIndex, NodeIndex, zig_ast_node_data_token_and_node),
        TokenAndToken = (TokenIndex, TokenIndex, zig_ast_node_data_token_and_token),
        OptNodeAndToken = (OptionalNodeIndex, TokenIndex, zig_ast_node_data_opt_node_and_token),
        OptTokenAndNode = (OptionalTokenIndex, NodeIndex, zig_ast_node_data_opt_token_and_node),
        OptTokenAndOptNode = (OptionalTokenIndex, OptionalNodeIndex, zig_ast_node_data_opt_token_and_opt_node),
        OptTokenAndOptToken = (OptionalTokenIndex, OptionalTokenIndex, zig_ast_node_data_opt_token_and_opt_token),
        ExtraAndFor = (ExtraIndex, For, zig_ast_node_data_for),
    }

    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct For(pub u32);

    impl For {
        pub fn inputs(self) -> u32 {
            self.0 & 0x7fffffff
        }

        pub fn has_else(self) -> bool {
            self.0 & 0x80000000 != 0
        }
    }

    impl From<u32> for For {
        fn from(value: u32) -> Self {
            Self(value)
        }
    }
}

pub mod extra_data {
    use super::*;

    pub trait ExtraData {
        const SIZE: u32;

        unsafe fn from(tree: *const Ast, index: ExtraIndex) -> Self;
    }

    impl ExtraData for u32 {
        const SIZE: u32 = 1;

        unsafe fn from(tree: *const Ast, index: ExtraIndex) -> Self {
            unsafe { *Ast::extra_data(tree).add(index.0 as usize) }
        }
    }

    macro_rules! extra_data_size {
        () => { 0u32 };
        ($head:ident, $($tail:ident,)*) => {
            extra_data_size!($($tail,)*) + 1u32
        };
    }

    macro_rules! extra_data_from {
        ($tree:ident, $index:expr,) => {};
        ($tree:ident, $index:expr, $head:ident, $($tail:ident,)*) => {
            let $head = unsafe { *Ast::extra_data($tree).add($index.0 as usize) };
            extra_data_from!($tree, ExtraIndex($index.0 + 1u32), $($tail,)*);
        };
    }

    macro_rules! extra_data {
        (
            $Data:ident,
            $(
                $(#[$($attrss:tt)*])*
                $field:ident: $Type:path,
            )+
        ) => {
            #[repr(C)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub struct $Data {
                $(
                    $(#[$($attrss)*])*
                    pub $field: $Type,
                )+
            }

            impl ExtraData for $Data {
                const SIZE: u32 = extra_data_size!($($field,)+);

                unsafe fn from(tree: *const Ast, index: ExtraIndex) -> Self {
                    extra_data_from!(tree, index, $($field,)+);
                    Self { $($field: $Type($field),)+ }
                }
            }
        };
    }

    extra_data! {
        LocalVarDecl,
        type_node: NodeIndex,
        align_node: NodeIndex,
    }

    extra_data! {
        ArrayTypeSentinel,
        sentinel: NodeIndex,
        elem_type: NodeIndex,
    }

    extra_data! {
        PtrType,
        sentinel: OptionalNodeIndex,
        align_node: OptionalNodeIndex,
        addrspace_node: OptionalNodeIndex,
    }

    extra_data! {
        PtrTypeBitRange,
        sentinel: OptionalNodeIndex,
        align_node: NodeIndex,
        addrspace_node: OptionalNodeIndex,
        bit_range_start: NodeIndex,
        bit_range_end: NodeIndex,
    }

    extra_data! {
        SubRange,
        /// NodeIndex into extra_data.
        start: ExtraIndex,
        /// NodeIndex into extra_data.
        end: ExtraIndex,
    }

    extra_data! {
        If,
        then_expr: NodeIndex,
        else_expr: NodeIndex,
    }

    extra_data! {
        ContainerField,
        align_expr: NodeIndex,
        value_expr: NodeIndex,
    }

    extra_data! {
        GlobalVarDecl,
        /// Populated if there is an explicit type ascription.
        type_node: OptionalNodeIndex,
        /// Populated if align(A) is present.
        align_node: OptionalNodeIndex,
        /// Populated if addrspace(A) is present.
        addrspace_node: OptionalNodeIndex,
        /// Populated if linksection(A) is present.
        section_node: OptionalNodeIndex,
    }

    extra_data! {
        Slice,
        start: NodeIndex,
        end: NodeIndex,
    }

    extra_data! {
        SliceSentinel,
        start: NodeIndex,
        /// May be .none if the slice is "open"
        end: OptionalNodeIndex,
        sentinel: NodeIndex,
    }

    extra_data! {
        While,
        cont_expr: OptionalNodeIndex,
        then_expr: NodeIndex,
        else_expr: NodeIndex,
    }

    extra_data! {
        WhileCont,
        cont_expr: NodeIndex,
        then_expr: NodeIndex,
    }

    extra_data! {
        FnProtoOne,
        /// Populated if there is exactly 1 parameter. Otherwise there are 0 parameters.
        param: OptionalNodeIndex,
        /// Populated if align(A) is present.
        align_expr: OptionalNodeIndex,
        /// Populated if addrspace(A) is present.
        addrspace_expr: OptionalNodeIndex,
        /// Populated if linksection(A) is present.
        section_expr: OptionalNodeIndex,
        /// Populated if callconv(A) is present.
        callconv_expr: OptionalNodeIndex,
    }

    extra_data! {
        FnProto,
        params_start: ExtraIndex,
        params_end: ExtraIndex,
        /// Populated if align(A) is present.
        align_expr: OptionalNodeIndex,
        /// Populated if addrspace(A) is present.
        addrspace_expr: OptionalNodeIndex,
        /// Populated if linksection(A) is present.
        section_expr: OptionalNodeIndex,
        /// Populated if callconv(A) is present.
        callconv_expr: OptionalNodeIndex,
    }

    impl FnProto {
        pub fn params(self) -> impl Iterator<Item = ExtraIndex> {
            ExtraIndexIterator::from_range(self.params_start, self.params_end)
        }
    }

    extra_data! {
        Asm,
        items_start: ExtraIndex,
        items_end: ExtraIndex,
        clobbers: OptionalNodeIndex,
        /// Needed to make lastToken() work.
        rparen: TokenIndex,
    }

    impl Asm {
        pub fn items(self) -> impl Iterator<Item = ExtraIndex> {
            ExtraIndexIterator::from_range(self.items_start, self.items_end)
        }
    }
}
