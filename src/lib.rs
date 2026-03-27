use std::ffi::CString;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

pub mod sys;
pub use sys::extra_data::*;
pub use sys::node_data::*;
pub use sys::*;

pub struct Ast {
    _source: CString,
    tree: *mut sys::Ast,
}

impl Drop for Ast {
    fn drop(&mut self) {
        unsafe { sys::Ast::deinit(self.tree) };
    }
}

impl Ast {
    pub fn parse<T: Into<Vec<u8>>>(source: T) -> Option<Self> {
        let cstr = CString::new(source).ok()?;
        let ptr = cstr.as_bytes_with_nul().as_ptr();
        let tree = unsafe { sys::Ast::parse(ptr) };
        assert!(ptr == unsafe { sys::Ast::source(tree) },);
        if tree.is_null() {
            None
        } else {
            Some(Self {
                _source: cstr,
                tree,
            })
        }
    }

    fn check_token_index(&self, index: TokenIndex) {
        let count = unsafe { sys::Ast::token_count(self.tree) };
        let index = index.0;
        assert!(
            index < count,
            "token index out of bounds: the token count is {count} but the index is {index}"
        );
    }

    fn check_node_index(&self, index: NodeIndex) {
        let count = unsafe { sys::Ast::node_count(self.tree) };
        let index = index.0;
        assert!(
            index < count,
            "node index out of bounds: the node count is {count} but the index is {index}"
        );
    }

    fn check_extra_range(&self, range: SubRange) {
        let len = unsafe { sys::Ast::extra_data_len(self.tree) };
        let start = range.start.0;
        let end = range.end.0;
        assert!(
            start <= end,
            "extra data range starts at {start} but ends at {end}"
        );
        assert!(
            end <= len,
            "extra data end index {end} out of range for extra data of length {len}"
        );
    }

    pub fn token_count(&self) -> u32 {
        unsafe { sys::Ast::token_count(self.tree) }
    }

    pub fn token_tag(&self, index: TokenIndex) -> TokenTag {
        self.check_token_index(index);
        unsafe { sys::Ast::token_tag(self.tree, index) }
    }

    pub fn node_count(&self) -> u32 {
        unsafe { sys::Ast::node_count(self.tree) }
    }

    pub fn node_tag(&self, index: NodeIndex) -> NodeTag {
        self.check_node_index(index);
        unsafe { sys::Ast::node_tag(self.tree, index) }
    }

    pub fn node_source(&self, index: NodeIndex) -> &[u8] {
        self.check_node_index(index);
        let mut len = MaybeUninit::uninit();
        unsafe {
            let ptr = sys::Ast::node_source(self.tree, index, len.as_mut_ptr());
            std::slice::from_raw_parts(ptr, len.assume_init())
        }
    }

    pub fn node_main_token(&self, index: NodeIndex) -> TokenIndex {
        self.check_node_index(index);
        unsafe { sys::Ast::node_main_token(self.tree, index) }
    }

    pub fn first_token(&self, index: NodeIndex) -> TokenIndex {
        self.check_node_index(index);
        unsafe { sys::Ast::first_token(self.tree, index) }
    }

    pub fn last_token(&self, index: NodeIndex) -> TokenIndex {
        self.check_node_index(index);
        unsafe { sys::Ast::last_token(self.tree, index) }
    }

    pub fn token_slice(&self, index: TokenIndex) -> &[u8] {
        self.check_token_index(index);
        let mut len = MaybeUninit::uninit();
        unsafe {
            let ptr = sys::Ast::token_slice(self.tree, index, len.as_mut_ptr());
            std::slice::from_raw_parts(ptr, len.assume_init())
        }
    }

    pub fn extra_data_slice<T: From<u32>>(&self, range: SubRange) -> impl Iterator<Item = T> {
        self.check_extra_range(range);
        ExtraDataSlice {
            tree: self,
            iter: ExtraIndexIterator::from_range(range.start, range.end),
            _marker: PhantomData,
        }
    }

    pub fn extra_data_slice_with_len<T: From<u32>>(
        &self,
        start: ExtraIndex,
        len: u32,
    ) -> impl Iterator<Item = T> {
        let end = ExtraIndex(start.0 + len);
        self.extra_data_slice(SubRange { start, end })
    }

    pub fn extra_data<T: ExtraData>(&self, index: ExtraIndex) -> T {
        let start = index;
        let end = ExtraIndex(start.0 + T::SIZE);
        self.check_extra_range(SubRange { start, end });
        unsafe { T::from(self.tree, index) }
    }

    pub unsafe fn node_data_unchecked<T: NodeData>(&self, index: NodeIndex) -> T {
        unsafe { T::from(self.tree, index) }
    }

    pub fn root_decls(&self) -> impl Iterator<Item = NodeIndex> {
        // Assumes Zig mode (not ZON)
        let extra_range: SubRange = unsafe { self.node_data_unchecked(NodeIndex::ROOT) };
        self.extra_data_slice(extra_range)
    }

    pub fn full_node<'ast, T: Full<'ast, 0>>(&'ast self, index: NodeIndex) -> Option<T> {
        self.check_node_index(index);
        let mut buffer = [NodeIndex(0); 0];
        unsafe { T::from(self.tree, &mut buffer, index) }
    }

    pub fn full_node_buffered<'ast, const N: usize, T: Full<'ast, N>>(
        &'ast self,
        index: NodeIndex,
    ) -> Option<BufferedData<'ast, N, T>> {
        const { assert!(N > 0, "use full_node() instead") };
        self.check_node_index(index);
        let mut buffer = Box::new_uninit();
        let data = unsafe { T::from(self.tree, buffer.as_mut_ptr(), index) };
        data.map(|data| BufferedData {
            _buffer: buffer,
            data,
            _marker: PhantomData,
        })
    }

    pub fn builtin_call_params<'ast>(
        &'ast self,
        index: NodeIndex,
    ) -> Option<BufferedSlice<'ast, 2>> {
        self.check_node_index(index);
        let mut buffer = Box::new_uninit();
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe {
            sys::Ast::builtin_call_params(self.tree, buffer.as_mut_ptr(), index, count.as_mut_ptr())
        };
        if ptr.is_null() {
            None
        } else {
            let slice = unsafe { std::slice::from_raw_parts(ptr, count.assume_init()) };
            Some(BufferedSlice {
                _buffer: buffer,
                slice,
            })
        }
    }

    pub fn builtin_call_tag(&self, index: NodeIndex) -> Option<BuiltinFnTag> {
        builtin_fn_tag(self.token_slice(self.node_main_token(index)))
    }

    pub fn block_statements<'ast>(&'ast self, index: NodeIndex) -> Option<BufferedSlice<'ast, 2>> {
        self.check_node_index(index);
        let mut buffer = Box::new_uninit();
        let mut count = MaybeUninit::uninit();
        let ptr = unsafe {
            sys::Ast::block_statements(self.tree, buffer.as_mut_ptr(), index, count.as_mut_ptr())
        };
        if ptr.is_null() {
            None
        } else {
            let slice = unsafe { std::slice::from_raw_parts(ptr, count.assume_init()) };
            Some(BufferedSlice {
                _buffer: buffer,
                slice,
            })
        }
    }

    pub fn parse_string_literal(&self, index: NodeIndex) -> Option<OwnedString> {
        parse_string_literal(self.token_slice(self.node_main_token(index)))
    }
}

pub struct BufferedData<'ast, const N: usize, T: Sized> {
    _buffer: Box<MaybeUninit<[NodeIndex; N]>>,
    data: T,
    _marker: PhantomData<&'ast ()>,
}

impl<'ast, const N: usize, T: Sized> BufferedData<'ast, N, T> {
    pub fn get(&'ast self) -> &'ast T {
        &self.data
    }
}

pub struct BufferedSlice<'ast, const N: usize> {
    _buffer: Box<MaybeUninit<[NodeIndex; N]>>,
    slice: &'ast [NodeIndex],
}

impl<'ast, const N: usize> BufferedSlice<'ast, N> {
    pub fn get(&'ast self) -> &'ast [NodeIndex] {
        self.slice
    }
}

struct ExtraDataSlice<'ast, T: From<u32>> {
    tree: &'ast Ast,
    iter: ExtraIndexIterator,
    _marker: PhantomData<T>,
}

impl<T: From<u32>> Iterator for ExtraDataSlice<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        match self.iter.next() {
            Some(index) => Some(T::from(self.tree.extra_data(index))),
            None => None,
        }
    }
}

pub fn builtin_fn_tag(name: &[u8]) -> Option<BuiltinFnTag> {
    let mut tag = MaybeUninit::uninit();
    unsafe {
        if sys::Ast::builtin_fn_tag(name.as_ptr(), name.len(), tag.as_mut_ptr()) {
            Some(tag.assume_init())
        } else {
            None
        }
    }
}

pub fn parse_string_literal(bytes: &[u8]) -> Option<OwnedString> {
    match bytes {
        [b'"', .., b'"'] => {}
        _ => return None,
    }
    let mut len = bytes.len();
    unsafe {
        let ptr = sys::Ast::parse_string_literal(bytes.as_ptr(), &mut len);
        if ptr.is_null() {
            None
        } else {
            Some(OwnedString { ptr, len })
        }
    }
}

pub struct OwnedString {
    ptr: *mut u8,
    len: usize,
}

impl OwnedString {
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_bytes_mut(&self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl AsRef<[u8]> for OwnedString {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for OwnedString {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_bytes_mut()
    }
}

impl Drop for OwnedString {
    fn drop(&mut self) {
        unsafe { sys::Ast::free_string(self.ptr, self.len) };
    }
}
