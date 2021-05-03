use super::*;

#[test]
fn test_to_string() {
    let mut son = Node::new(Token::from_str0("son"), "child");
    son.push_node_vec("grand_children", vec![
        Node::new(Token::from_str0("grand_child1"), "grand_child"),
        Node::new(Token::from_str0("grand_child2"), "grand_child"),
    ]);
    let mut daughter = Node::new(Token::from_str0("daughter"), "child");
    daughter.push_node_vec("grand_children", vec![
        Node::new(Token::from_str0("grand_child3"), "grand_child"),
    ]);
    let mut parent = Node::new(Token::from_str0("parent"), "parent");
    parent.push_node("son", son);
    parent.push_node("daughter", daughter);
    let res = format!("{}", parent.to_string());

    println!("{}", res);
    assert_eq!(res, "\
self: parent (parent)
daughter:
  self: daughter (child)
  grand_children:
  - self: grand_child3 (grand_child)
son:
  self: son (child)
  grand_children:
  - self: grand_child1 (grand_child)
  - self: grand_child2 (grand_child)");
}
