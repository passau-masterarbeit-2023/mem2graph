#[cfg(test)]
use crate::graph_structs::*;
#[cfg(test)]
fn create_test_structs() -> Vec<Node> {

    let data_structure_node = Node::DataStructureNode(DataStructureNode {
        addr: 0,
        byte_size: 8,
        nb_pointer_nodes: 0,
        nb_value_nodes: 0,
        dtn_type : DtnTypes::Basestruct
    });

    let base_value_node = ValueNode::BaseValueNode(BaseValueNode {
        addr: 1,
        value: [0, 1, 2, 3, 4, 5, 6, 7],
        dtn_addr: 0,
    });

    let base_pointer_node = PointerNode::BasePointerNode(BasePointerNode {
        addr: 2,
        points_to: 8,
        dtn_addr: 0,
    });

    let key_data = KeyData {
        addr: 3,
        name: "key1".to_string(),
        key: vec![0, 1, 2, 3, 4, 5, 6, 7],
        len: 4,
        real_len: 4,
    };

    let key_node = ValueNode::KeyNode(KeyNode {
        addr: 4,
        dtn_addr: 0,
        value: [0, 1, 2, 3, 4, 5, 6, 7],
        key: vec![0, 1, 2, 3, 4, 5, 6, 7],
        key_data,
    });

    let nodes: Vec<Node> = vec![
        data_structure_node,
        // enum of enum
        Node::ValueNode(base_value_node.clone()),
        Node::ValueNode(key_node.clone()),
        Node::PointerNode(base_pointer_node.clone()),
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
                }
            }
        }
    }
    assert_eq!(counter_data_structure_nodes, 1);
    assert_eq!(counter_value_nodes, 2);
    assert_eq!(counter_pointer_nodes, 1);
    
}

#[test]
fn test_important_nodes() {
    crate::tests::setup();
    // test with Vec<Node>
    let nodes = create_test_structs();

    let mut counter_importants = 0;
    for node in nodes {
        if node.is_important() {
            counter_importants += 1;
        }
    }
    assert_eq!(counter_importants, 1);
}

#[test]
fn test_is_pointer() {
    crate::tests::setup();

    // test with Vec<Node>
    let nodes = create_test_structs();

    let mut counter_pointers = 0;
    for node in nodes {
        if node.is_pointer() {
            counter_pointers += 1;
        }
    }
    assert_eq!(counter_pointers, 1);
}

#[test]
fn test_is_value() {
    crate::tests::setup();

    // test with Vec<Node>
    let nodes = create_test_structs();

    let mut counter_values = 0;
    for node in nodes {
        if node.is_value() {
            counter_values += 1;
        }
    }
    assert_eq!(counter_values, 2);
}

#[test]
fn test_debug() {
    crate::tests::setup();

    // test with Vec<Node>
    let nodes = create_test_structs();

    for node in nodes {
        log::debug!("testin the debug fmt for node {:?}", node);
    }
}