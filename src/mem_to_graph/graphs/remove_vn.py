import sys
import os

INPUT_DIR = "graphs/graphs"
VN_CLEANED_DIR = "graphs/graphs_no_vn"
GRAPH_VIZ_DIR = "graphs/graph_viz"

def graphs_to_clean():
    """
    This function returns a list of all the graphs in the INPUT_DIR directory.
    """
    graphs = []

    print("    INPUT:", os.listdir(INPUT_DIR))

    for graph in os.listdir(INPUT_DIR):
        graphs.append(graph)

    return graphs

def generate_graph_viz(input_file, output_dir):
    """
    Call sfdp to generate a graphviz file from a graph file.
    Ex: sfdp -Gsize=67! -Goverlap=prism -Tpng tests/graphs_no_vn/test_graph_from_302-1644391327_no_vn.gv > tests/graph_viz/test_graph_from_302-1644391327_no_vn-sfdp.png
    """
    # Make sure the output directory exists
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Generate the graphviz file
    output_file = os.path.join(output_dir, os.path.basename(input_file)).replace(".gv", "-sfdp.png")
    os.system(f"sfdp -Gsize=67! -Goverlap=prism -Tpng {input_file} > {output_file}")

    print(f"Graphviz file saved as {output_file}")


def remove_vn_lines(input_file, output_dir):
    # Make sure the output directory exists
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Read input file and remove lines containing "VN"
    with open(input_file, 'r') as f:
        lines = f.readlines()
        
        # is node line
        def is_node_line(line):
            return "->" not in line
        
        # get the specials nodes addresses to identify the lines to keep
        specials_nodes_lines = [line for line in lines if is_node_line(line) and "label" in line]
        specials_nodes_addr = [line.split("(")[1] for line in specials_nodes_lines]
        specials_nodes_addr = [line.split(")")[0] for line in specials_nodes_addr]
        print("Specials nodes addresses:", specials_nodes_addr)

        def is_special_node_line(line):
            for addr in specials_nodes_addr:
                if addr in line:
                    return True
            return False
        
        def is_vn_line(line):
            return "VN" in line

        filtered_lines = [line for line in lines if not is_vn_line(line) or is_special_node_line(line)]

    # Save the new file in the output directory
    output_file = os.path.join(output_dir, os.path.basename(input_file)).replace(".gv", "_no_vn.gv")
    with open(output_file, 'w') as f:
        f.writelines(filtered_lines)

    print(f"Filtered file saved as {output_file}")

if __name__ == "__main__":
    graphs = graphs_to_clean()

    if len(graphs) == 0:
        print("All graphs have already been cleaned.")
        sys.exit(0)

    for graph in graphs:
        input_file = os.path.join(INPUT_DIR, graph)
        remove_vn_lines(input_file, VN_CLEANED_DIR)
    
    # Generate graphviz files
    graph_no_vn = os.listdir(VN_CLEANED_DIR)
    for graph in graph_no_vn:
        input_file = os.path.join(VN_CLEANED_DIR, graph)
        generate_graph_viz(input_file, GRAPH_VIZ_DIR)
    
