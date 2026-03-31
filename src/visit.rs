use super::*;

pub trait Visit {
    fn visit(&mut self, tree: &Ast, index: NodeIndex) {
        visit(self, tree, index);
    }

    fn visit_optional(&mut self, tree: &Ast, opt_index: OptionalNodeIndex) {
        visit_optional(self, tree, opt_index);
    }

    fn visit_slice(&mut self, tree: &Ast, indexes: &[NodeIndex]) {
        visit_slice(self, tree, indexes);
    }

    fn visit_iterator<I: IntoIterator<Item = NodeIndex>>(&mut self, tree: &Ast, iterator: I) {
        visit_iterator(self, tree, iterator);
    }
}

pub fn visit_optional<V>(visitor: &mut V, tree: &Ast, opt_index: OptionalNodeIndex)
where
    V: Visit + ?Sized,
{
    if let Some(index) = opt_index.to_option() {
        visitor.visit(tree, index);
    }
}

pub fn visit_slice<V>(visitor: &mut V, tree: &Ast, indexes: &[NodeIndex])
where
    V: Visit + ?Sized,
{
    for &index in indexes {
        visitor.visit(tree, index);
    }
}

pub fn visit_iterator<V, I>(visitor: &mut V, tree: &Ast, iterator: I)
where
    V: Visit + ?Sized,
    I: IntoIterator<Item = NodeIndex>,
{
    for index in iterator.into_iter() {
        visitor.visit(tree, index);
    }
}

pub fn visit<V>(visitor: &mut V, tree: &Ast, index: NodeIndex)
where
    V: Visit + ?Sized,
{
    match tree.node_tag(index) {
        NodeTag::Root => {
            // Assumes Zig mode (not ZON)
            visitor.visit_iterator(tree, tree.root_decls());
        }
        NodeTag::GlobalVarDecl
        | NodeTag::LocalVarDecl
        | NodeTag::SimpleVarDecl
        | NodeTag::AlignedVarDecl => {
            let var_decl: full::VarDecl = tree.full_node(index).unwrap();
            visitor.visit_optional(tree, var_decl.ast.type_node);
            visitor.visit_optional(tree, var_decl.ast.align_node);
            visitor.visit_optional(tree, var_decl.ast.addrspace_node);
            visitor.visit_optional(tree, var_decl.ast.section_node);
            visitor.visit_optional(tree, var_decl.ast.init_node);
        }
        NodeTag::AssignDestructure => {
            let assign_destructure: full::AssignDestructure = tree.full_node(index).unwrap();
            visitor.visit_slice(tree, assign_destructure.ast.variables());
            visitor.visit(tree, assign_destructure.ast.value_expr);
        }
        NodeTag::ArrayType | NodeTag::ArrayTypeSentinel => {
            let array_type: full::ArrayType = tree.full_node(index).unwrap();
            visitor.visit(tree, array_type.ast.elem_count);
            visitor.visit_optional(tree, array_type.ast.sentinel);
            visitor.visit(tree, array_type.ast.elem_type);
        }
        NodeTag::PtrTypeAligned
        | NodeTag::PtrTypeSentinel
        | NodeTag::PtrType
        | NodeTag::PtrTypeBitRange => {
            let ptr_type: full::PtrType = tree.full_node(index).unwrap();
            visitor.visit_optional(tree, ptr_type.ast.align_node);
            visitor.visit_optional(tree, ptr_type.ast.addrspace_node);
            visitor.visit_optional(tree, ptr_type.ast.sentinel);
            visitor.visit_optional(tree, ptr_type.ast.bit_range_start);
            visitor.visit_optional(tree, ptr_type.ast.bit_range_end);
            visitor.visit(tree, ptr_type.ast.child_type);
        }
        NodeTag::SliceOpen | NodeTag::Slice | NodeTag::SliceSentinel => {
            let slice: full::Slice = tree.full_node(index).unwrap();
            visitor.visit(tree, slice.ast.sliced);
            visitor.visit(tree, slice.ast.start);
            visitor.visit_optional(tree, slice.ast.end);
            visitor.visit_optional(tree, slice.ast.sentinel);
        }
        NodeTag::ArrayInitOne
        | NodeTag::ArrayInitOneComma
        | NodeTag::ArrayInitDotTwo
        | NodeTag::ArrayInitDotTwoComma
        | NodeTag::ArrayInitDot
        | NodeTag::ArrayInitDotComma
        | NodeTag::ArrayInit
        | NodeTag::ArrayInitComma => {
            let buffered = tree.full_node_buffered(index).unwrap();
            let array_init: &full::ArrayInit = buffered.get();
            visitor.visit_slice(tree, array_init.ast.elements());
            visitor.visit_optional(tree, array_init.ast.type_expr);
        }
        NodeTag::StructInitOne
        | NodeTag::StructInitOneComma
        | NodeTag::StructInitDotTwo
        | NodeTag::StructInitDotTwoComma
        | NodeTag::StructInitDot
        | NodeTag::StructInitDotComma
        | NodeTag::StructInit
        | NodeTag::StructInitComma => {
            let buffered = tree.full_node_buffered(index).unwrap();
            let struct_init: &full::StructInit = buffered.get();
            visitor.visit_slice(tree, struct_init.ast.fields());
            visitor.visit_optional(tree, struct_init.ast.type_expr);
        }
        NodeTag::CallOne | NodeTag::CallOneComma | NodeTag::Call | NodeTag::CallComma => {
            let buffered = tree.full_node_buffered(index).unwrap();
            let call: &full::Call = buffered.get();
            visitor.visit(tree, call.ast.fn_expr);
            visitor.visit_slice(tree, call.ast.params());
        }
        NodeTag::Switch | NodeTag::SwitchComma => {
            let switch: full::Switch = tree.full_node(index).unwrap();
            visitor.visit(tree, switch.ast.condition);
            visitor.visit_slice(tree, switch.ast.cases());
        }
        NodeTag::SwitchCaseOne
        | NodeTag::SwitchCaseInlineOne
        | NodeTag::SwitchCase
        | NodeTag::SwitchCaseInline => {
            let switch_case: full::SwitchCase = tree.full_node(index).unwrap();
            visitor.visit_slice(tree, switch_case.ast.values());
            visitor.visit(tree, switch_case.ast.target_expr);
        }
        NodeTag::WhileSimple | NodeTag::WhileCont | NodeTag::While => {
            let full_while: full::While = tree.full_node(index).unwrap();
            visitor.visit(tree, full_while.ast.cond_expr);
            visitor.visit_optional(tree, full_while.ast.cont_expr);
            visitor.visit(tree, full_while.ast.then_expr);
            visitor.visit_optional(tree, full_while.ast.else_expr);
        }
        NodeTag::ForSimple | NodeTag::For => {
            let full_for: full::For = tree.full_node(index).unwrap();
            visitor.visit_slice(tree, full_for.ast.inputs());
            visitor.visit(tree, full_for.ast.then_expr);
            visitor.visit_optional(tree, full_for.ast.else_expr);
        }
        NodeTag::IfSimple | NodeTag::If => {
            let full_if: full::If = tree.full_node(index).unwrap();
            visitor.visit(tree, full_if.ast.cond_expr);
            visitor.visit(tree, full_if.ast.then_expr);
            visitor.visit_optional(tree, full_if.ast.else_expr);
        }
        NodeTag::FnProtoSimple | NodeTag::FnProtoMulti | NodeTag::FnProtoOne | NodeTag::FnProto => {
            let buffered = tree.full_node_buffered(index).unwrap();
            let fn_proto: &full::FnProto = buffered.get();
            visitor.visit_optional(tree, fn_proto.ast.return_type);
            visitor.visit_slice(tree, fn_proto.ast.params());
            visitor.visit_optional(tree, fn_proto.ast.align_expr);
            visitor.visit_optional(tree, fn_proto.ast.addrspace_expr);
            visitor.visit_optional(tree, fn_proto.ast.section_expr);
            visitor.visit_optional(tree, fn_proto.ast.callconv_expr);
        }
        NodeTag::BuiltinCallTwo
        | NodeTag::BuiltinCallTwoComma
        | NodeTag::BuiltinCall
        | NodeTag::BuiltinCallComma => {
            let buffered = tree.builtin_call_params(index).unwrap();
            let params = buffered.get();
            visitor.visit_slice(tree, params);
        }
        NodeTag::ContainerDecl
        | NodeTag::ContainerDeclTrailing
        | NodeTag::ContainerDeclTwo
        | NodeTag::ContainerDeclTwoTrailing
        | NodeTag::ContainerDeclArg
        | NodeTag::ContainerDeclArgTrailing
        | NodeTag::TaggedUnion
        | NodeTag::TaggedUnionTrailing
        | NodeTag::TaggedUnionTwo
        | NodeTag::TaggedUnionTwoTrailing
        | NodeTag::TaggedUnionEnumTag
        | NodeTag::TaggedUnionEnumTagTrailing => {
            let buffered = tree.full_node_buffered(index).unwrap();
            let container_decl: &full::ContainerDecl = buffered.get();
            visitor.visit_slice(tree, container_decl.ast.members());
            visitor.visit_optional(tree, container_decl.ast.arg);
        }
        NodeTag::ContainerFieldInit | NodeTag::ContainerFieldAlign | NodeTag::ContainerField => {
            let container_field: full::ContainerField = tree.full_node(index).unwrap();
            visitor.visit_optional(tree, container_field.ast.type_expr);
            visitor.visit_optional(tree, container_field.ast.align_expr);
            visitor.visit_optional(tree, container_field.ast.value_expr);
        }
        NodeTag::BlockTwo
        | NodeTag::BlockTwoSemicolon
        | NodeTag::Block
        | NodeTag::BlockSemicolon => {
            let buffered = tree.block_statements(index).unwrap();
            let statements = buffered.get();
            visitor.visit_slice(tree, statements);
        }
        NodeTag::AsmSimple | NodeTag::Asm => {
            let asm: full::Asm = tree.full_node(index).unwrap();
            visitor.visit_slice(tree, asm.outputs());
            visitor.visit_slice(tree, asm.inputs());
            visitor.visit(tree, asm.ast.template);
            visitor.visit_slice(tree, asm.ast.items());
            visitor.visit_optional(tree, asm.ast.clobbers);
        }
        NodeTag::Defer
        | NodeTag::Deref
        | NodeTag::Suspend
        | NodeTag::Resume
        | NodeTag::Comptime
        | NodeTag::Nosuspend
        | NodeTag::BoolNot
        | NodeTag::Negation
        | NodeTag::BitNot
        | NodeTag::NegationWrap
        | NodeTag::AddressOf
        | NodeTag::Try
        | NodeTag::OptionalType => {
            let expr: NodeIndex = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, expr);
        }
        NodeTag::Return => {
            let expr: OptionalNodeIndex = unsafe { tree.node_data_unchecked(index) };
            visitor.visit_optional(tree, expr);
        }
        NodeTag::Catch
        | NodeTag::EqualEqual
        | NodeTag::BangEqual
        | NodeTag::LessThan
        | NodeTag::GreaterThan
        | NodeTag::LessOrEqual
        | NodeTag::GreaterOrEqual
        | NodeTag::AssignMul
        | NodeTag::AssignDiv
        | NodeTag::AssignMod
        | NodeTag::AssignAdd
        | NodeTag::AssignSub
        | NodeTag::AssignShl
        | NodeTag::AssignShlSat
        | NodeTag::AssignShr
        | NodeTag::AssignBitAnd
        | NodeTag::AssignBitXor
        | NodeTag::AssignBitOr
        | NodeTag::AssignMulWrap
        | NodeTag::AssignAddWrap
        | NodeTag::AssignSubWrap
        | NodeTag::AssignMulSat
        | NodeTag::AssignAddSat
        | NodeTag::AssignSubSat
        | NodeTag::Assign
        | NodeTag::MergeErrorSets
        | NodeTag::Mul
        | NodeTag::Div
        | NodeTag::Mod
        | NodeTag::ArrayMult
        | NodeTag::MulWrap
        | NodeTag::MulSat
        | NodeTag::Add
        | NodeTag::Sub
        | NodeTag::ArrayCat
        | NodeTag::AddWrap
        | NodeTag::SubWrap
        | NodeTag::AddSat
        | NodeTag::SubSat
        | NodeTag::Shl
        | NodeTag::ShlSat
        | NodeTag::Shr
        | NodeTag::BitAnd
        | NodeTag::BitXor
        | NodeTag::BitOr
        | NodeTag::Orelse
        | NodeTag::BoolAnd
        | NodeTag::BoolOr
        | NodeTag::ArrayAccess
        | NodeTag::SwitchRange
        | NodeTag::FnDecl
        | NodeTag::ErrorUnion => {
            let NodeAndNode(lhs, rhs) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, lhs);
            visitor.visit(tree, rhs);
        }
        NodeTag::ForRange => {
            let NodeAndOptNode(lhs, rhs) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, lhs);
            visitor.visit_optional(tree, rhs);
        }
        NodeTag::AsmLegacy => {
            let NodeAndExtra(lhs, _) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, lhs);
        }
        NodeTag::FieldAccess
        | NodeTag::UnwrapOptional
        | NodeTag::GroupedExpression
        | NodeTag::AsmInput => {
            let NodeAndToken(lhs, _) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, lhs);
        }
        NodeTag::AnyframeType => {
            let TokenAndNode(_, rhs) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, rhs);
        }
        NodeTag::AsmOutput => {
            let OptNodeAndToken(lhs, _) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit_optional(tree, lhs);
        }
        NodeTag::TestDecl | NodeTag::Errdefer => {
            let OptTokenAndNode(_, rhs) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit(tree, rhs);
        }
        NodeTag::Continue | NodeTag::Break => {
            let OptTokenAndOptNode(_, rhs) = unsafe { tree.node_data_unchecked(index) };
            visitor.visit_optional(tree, rhs);
        }
        NodeTag::AnyframeLiteral => {}
        NodeTag::CharLiteral => {}
        NodeTag::NumberLiteral => {}
        NodeTag::UnreachableLiteral => {}
        NodeTag::Identifier => {}
        NodeTag::EnumLiteral => {}
        NodeTag::StringLiteral => {}
        NodeTag::MultilineStringLiteral => {}
        NodeTag::ErrorSetDecl => {}
        NodeTag::ErrorValue => {}
    }
}
