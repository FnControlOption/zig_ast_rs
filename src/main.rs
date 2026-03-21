#![allow(dead_code)]

use zig_ast::*;

fn main() {
    let source = docstr::docstr!(
        /// pub fn main() void {}
        /// const foo = 42;
        /// fn bar(baz: []const u8) void {
        ///     const fizz, const buzz = .{ 6, 7 };
        /// }
    );
    let tree = Ast::parse(source).unwrap();
    println!("{:?}", tree.node_tag(0.into()));
    let range: SubRange = unsafe { tree.node_data_unchecked(0.into()) };
    println!("{range:?}");
    for index in tree.extra_data_slice(range) {
        println!();
        let tag = tree.node_tag(index);
        println!("{tag:?}");
        match tag {
            NodeTag::FnDecl => {
                let NodeAndNode(lhs, rhs) = unsafe { tree.node_data_unchecked(index) };
                println!("{:?}", tree.node_tag(lhs));
                println!("{:?}", tree.node_tag(rhs));

                let buffered = tree.full_node_buffered(index).unwrap();
                let fn_proto: &full::FnProto = buffered.get();
                for &param in fn_proto.ast.params() {
                    println!("{:?}", tree.node_tag(param));
                }

                match tree.node_tag(rhs) {
                    NodeTag::BlockTwoSemicolon => {
                        let OptNodeAndOptNode(lhs, rhs) = unsafe { tree.node_data_unchecked(rhs) };
                        println!("{:?}", lhs.into_option().map(|index| tree.node_tag(index)));
                        println!("{:?}", rhs.into_option().map(|index| tree.node_tag(index)));
                        match lhs.into_option() {
                            Some(index) => match tree.node_tag(index) {
                                NodeTag::AssignDestructure => {
                                    let assign_destructure: full::AssignDestructure =
                                        tree.full_node(index).unwrap();
                                    println!("{:?}", assign_destructure.ast.equal_token);
                                }
                                _ => {}
                            },
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
            NodeTag::SimpleVarDecl => {
                let OptNodeAndOptNode(lhs, rhs) = unsafe { tree.node_data_unchecked(index) };
                println!("{:?}", lhs.into_option().map(|index| tree.node_tag(index)));
                println!("{:?}", rhs.into_option().map(|index| tree.node_tag(index)));

                let var_decl: full::VarDecl = tree.full_node(index).unwrap();
                println!("{:?}", var_decl.visib_token);
            }
            _ => {}
        }
    }
}
