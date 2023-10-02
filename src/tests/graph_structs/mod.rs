#[cfg(test)]
use crate::graph_structs::*;
#[cfg(test)]
fn create_test_structs() -> Vec<Node> {

    let data_structure_node = Node::ChunkHeaderNode(ChunkHeaderNode {
        addr: 0,
        byte_size: 8,
        flags: HeaderFlags{p : true, m : false, a : false},
        is_free: false,
        nb_pointer_nodes: 0,
        nb_value_nodes: 0
    });

    let base_value_node = Node::ValueNode(ValueNode {
        addr: 1,
        value: [0, 1, 2, 3, 4, 5, 6, 7],
        chn_addr: 0,
    });

    let base_pointer_node = Node::PointerNode(PointerNode {
        addr: 2,
        points_to: 8,
        chn_addr: 0,
    });

    let key_data = KeyData {
        addr: 3,
        name: "key1".to_string(),
        key: vec![0, 1, 2, 3, 4, 5, 6, 7],
        len: 4,
        real_len: 4,
    };

    let key_node = Node::KeyNode(KeyNode {
        addr: 4,
        chn_addr: 0,
        value: [0, 1, 2, 3, 4, 5, 6, 7],
        key: vec![0, 1, 2, 3, 4, 5, 6, 7],
        key_data,
    });

    let nodes: Vec<Node> = vec![
        data_structure_node,
        base_value_node.clone(),
        key_node.clone(),
        base_pointer_node.clone(),
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
            Node::ChunkHeaderNode(_) => {
                counter_data_structure_nodes += 1;
            }
            Node::ValueNode(_) => {
                counter_value_nodes += 1;
            }
            Node::KeyNode(_) => {
                counter_value_nodes += 1;
            }
            Node::PointerNode(_) => {
                counter_pointer_nodes += 1;
            }
            Node::FooterNode(_) => {}
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