use crate::graph_structs::*;

fn create_test_structs() -> Vec<Node> {
    let data_structure_node = Node::DataStructureNode(DataStructureNode {
        addr: 0,
        byte_size: 8,
        nb_pointer_nodes: 0,
        nb_value_nodes: 0,
        color: "blue".to_string(),
        style: "".to_string(),
    });

    let base_value_node = ValueNode::BaseValueNode(BaseValueNode {
            addr: 0,
            value: [0, 1, 2, 3, 4, 5, 6, 7],
            color: "grey".to_string(),
        }
    );

    let base_pointer_node = PointerNode::BasePointerNode(BasePointerNode {
        addr: 0,
        points_to: 8,
        color: "orange".to_string(),
    });

    let session_state_node = PointerNode::SessionStateNode(SessionStateNode {
        addr: 0,
        points_to: 16,
        color: "red".to_string(),
    });

    let ssh_struct_node = PointerNode::SshStructNode(SshStructNode {
        addr: 0,
        points_to: 24,
        color: "purple".to_string(),
    });

    let key_data = KeyData {
        name: "key1".to_string(),
        key: [0, 1, 2, 3, 4, 5, 6, 7],
        addr: [0, 1, 2, 3, 4, 5, 6, 7],
        len: 4,
        real_len: 4,
    };

    let key_node = ValueNode::KeyNode(KeyNode {
        addr: 0,
        value: [0, 1, 2, 3, 4, 5, 6, 7],
        key: [0, 1, 2, 3, 4, 5, 6, 7],
        key_data,
        color: "green".to_string(),
    });

    let nodes: Vec<Node> = vec![
        data_structure_node,
        // enum of enum
        Node::ValueNode(base_value_node.clone()),
        Node::ValueNode(key_node.clone()),
        Node::PointerNode(base_pointer_node.clone()),
        Node::PointerNode(session_state_node.clone()),
        Node::PointerNode(ssh_struct_node.clone()),
    ];

    nodes
}

#[test]
fn test_node_display() {
    crate::tests::setup();
    let nodes = create_test_structs();

    for node in nodes {
        println!("{}", node);
    }
}


#[test]
fn test_hierarchy() {
    crate::tests::setup();
    let nodes = create_test_structs();

    let mut counter_data_structure_nodes = 0;
    let mut counter_value_nodes = 0;
    let mut counter_pointer_nodes = 0;
    for node in nodes {
        match node {
            Node::DataStructureNode(_) => {
                counter_data_structure_nodes += 1;
            }
            Node::ValueNode(value_node) => {
                match value_node {
                    ValueNode::BaseValueNode(_) => {
                        counter_value_nodes += 1;
                    }
                    ValueNode::KeyNode(_) => {
                        counter_value_nodes += 1;
                    }
                }
            }
            Node::PointerNode(pointer_node) => {
                match pointer_node {
                    PointerNode::BasePointerNode(_) => {
                        counter_pointer_nodes += 1;
                    }
                    PointerNode::SessionStateNode(_) => {
                        counter_pointer_nodes += 1;
                    }
                    PointerNode::SshStructNode(_) => {
                        counter_pointer_nodes += 1;
                    }
                }
            }
            _ => panic!("Expected a Node enum"),
        }
    }
    assert_eq!(counter_data_structure_nodes, 1);
    assert_eq!(counter_value_nodes, 2);
    assert_eq!(counter_pointer_nodes, 3);
    
}

#[test]
fn test_important_nodes() {
    crate::tests::setup();
    // test with Vec<Node>
    let nodes = create_test_structs();

    let mut counter_importants = 0;
    for node in nodes {
        // if node.is_important() {
        //     counter_importants += 1;
        // }
        if node.is_important() {
            counter_importants += 1;
        }
    }
    assert_eq!(counter_importants, 3);
}
