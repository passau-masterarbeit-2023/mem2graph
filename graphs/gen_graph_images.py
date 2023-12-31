import sys
import os
import re

INPUT_DIR = "graphs/graphs"
VN_CLEANED_DIR = "graphs/graphs_cleaned"
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


def remove_and_clean_lines(input_file, output_dir):
    # Make sure the output directory exists
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)

    # Read input file and remove lines containing "VN"
    with open(input_file, 'r') as f:
        lines = f.readlines()

        important_addresses = set()

        def is_important_address_in_line(line):
            for important_addr in important_addresses:
                if important_addr in line:
                    return True
            return False
        
        LIST_ITEMS_TO_KEEP = ["KEY", "Ssh", "SST"]
        def is_line_to_skip(line):
            for item_to_keep in LIST_ITEMS_TO_KEEP:
                if item_to_keep in line:
                    important_addr = line.strip().split(" ")[0].replace("\"", "")
                    important_addresses.add(
                        important_addr
                    )
                    return False
            
            return "VN" in line and not is_important_address_in_line(line)

        filtered_lines = [line for line in lines if not is_line_to_skip(line)]

        # now, remove any line that starts with the word 'comment'
        filtered_lines = [line for line in filtered_lines if not line.strip().startswith("comment")]

        # now, in each line, remove the comment field of nodes
        for filtered_line in filtered_lines:
            line_no_comment = re.sub(r' comment=".*?"', '', filtered_line)

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
        remove_and_clean_lines(input_file, VN_CLEANED_DIR)
    
    # Generate graphviz files
    graph_no_vn = os.listdir(VN_CLEANED_DIR)
    for graph in graph_no_vn:
        input_file = os.path.join(VN_CLEANED_DIR, graph)
        generate_graph_viz(input_file, GRAPH_VIZ_DIR)
    
