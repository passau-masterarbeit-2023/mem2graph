from mem_to_graph.mem_to_graph import MemGraph

graph = MemGraph("dummy_file_path")  # File path is not used in this example
dot_output = graph.output_dot()
print(dot_output)

# ########## Output: ##########
# (phdtrack-1) [onyr@kenzael mem_to_graph]$ python example.py 
# digraph {
#     0 [ label = "\"A\"" ]
#     1 [ label = "\"B\"" ]
#     2 [ label = "\"C\"" ]
#     0 -> 1 [ ]
#     1 -> 2 [ ]
#     2 -> 0 [ ]
# }

